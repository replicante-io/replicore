use std::collections::HashSet;

use chrono::DateTime;
use chrono::Utc;
use failure::ResultExt;
use opentracingrust::Span;
use slog::debug;

use replicante_models_core::actions::Action;
use replicante_models_core::actions::ActionHistory;
use replicante_models_core::actions::ActionHistoryOrigin;
use replicante_models_core::actions::ActionState;
use replicante_models_core::events::action::ActionEvent;
use replicante_models_core::events::action::ActionHistory as ActionHistoryEvent;

use crate::follower::Follower;
use crate::ErrorKind;
use crate::Result;

/// Extract and persist action information.
pub fn process(follower: &Follower, event: &ActionEvent, span: Option<&mut Span>) -> Result<()> {
    match event {
        ActionEvent::Changed(info) => persist_action(follower, &info.current, span),
        ActionEvent::Finished(action) => persist_action(follower, action, span),
        ActionEvent::History(info) => process_history(follower, info, span),
        ActionEvent::Lost(action) => persist_action(follower, action, span),
        ActionEvent::New(action) => persist_action(follower, action, span),
    }
}

/// Helper function to persist an action to the viwe store.
fn persist_action(follower: &Follower, action: &Action, span: Option<&mut Span>) -> Result<()> {
    follower
        .store
        .persist()
        .action(
            action.clone(),
            span.as_ref().map(|span| span.context().clone()),
        )
        .with_context(|_| ErrorKind::StoreWrite("action"))?;
    Ok(())
}

/// Process an action history to synchronize the core and agent records.
fn process_history(
    follower: &Follower,
    info: &ActionHistoryEvent,
    span: Option<&mut Span>,
) -> Result<()> {
    let action_id = info.action_id;
    let cluster_id = info.cluster_id.clone();
    let finished_ts = info.finished_ts;
    let history = info.history.clone();
    let node_id = info.node_id.clone();
    let span_context = span.map(|span| span.context().clone());
    debug!(follower.logger, "Sync action history in view DB"; "action" => %action_id);

    // Insert any new action history records.
    // History records are append only (with the exception of the finished_ts field)
    // so we either have them and ignore them or insert them.
    let known_history: HashSet<(DateTime<Utc>, ActionState)> = follower
        .store
        .actions(cluster_id.clone())
        .history(action_id, span_context.clone())
        .with_context(|_| ErrorKind::StoreRead("action history transitions"))?
        .into_iter()
        .filter(|history| history.origin == ActionHistoryOrigin::Agent)
        .map(|history| (history.timestamp, history.state))
        .collect();
    let history: Vec<_> = history
        .into_iter()
        .filter(|history| {
            !known_history.contains(&(history.timestamp, history.state.clone().into()))
        })
        .map(|history| {
            ActionHistory::new(
                cluster_id.clone(),
                node_id.to_string(),
                history,
                ActionHistoryOrigin::Agent,
            )
        })
        .collect();
    if !history.is_empty() {
        follower
            .store
            .persist()
            .action_history(history, span_context.clone())
            .with_context(|_| ErrorKind::StoreWrite("action history transitions"))?;
    }

    // If finished, set finished_ts on all history items so they can be cleaned up.
    if let Some(finished_ts) = finished_ts {
        follower
            .store
            .actions(cluster_id)
            .finish_history(action_id, finished_ts, span_context)
            .with_context(|_| ErrorKind::StoreWrite("history finish timestamp"))?;
    }
    Ok(())
}
