use clap::ArgMatches;
use failure::ResultExt;
use reqwest::blocking::Client as ReqwestClient;
use reqwest::StatusCode;
use serde_json::Value;
use slog::debug;
use slog::info;
use slog::Logger;
use uuid::Uuid;

use replicante_models_core::api::apply::ApplyObject;
use replicante_models_core::api::validate::ErrorsCollection;
use replicante_util_failure::SerializableFail;

use crate::sso::Session;
use crate::sso::SessionStore;
use crate::ErrorKind;
use crate::Result;

const ENDPOINT_APPLY: &str = "/api/unstable/core/apply";
const ENDPOINT_CLUSTER: &str = "/api/unstable/core/cluster";
const ENDPOINT_CLUSTER_ACTION: &str = "/action";
const ENDPOINT_CLUSTER_ACTION_APPROVE: &str = "/approve";
const ENDPOINT_CLUSTER_ACTION_DISAPPROVE: &str = "/disapprove";
const ENDPOINT_CLUSTER_REFRESH: &str = "/refresh";

/// Replicante Core API client.
pub struct RepliClient<'a> {
    logger: &'a Logger,
    session: Session,
}

impl<'a> RepliClient<'a> {
    /// Approve a PENDING_APPROVE action so it can be scheduled.
    pub fn action_approve(&self, cluster: &str, action: Uuid) -> Result<()> {
        let client = ReqwestClient::builder()
            .build()
            .with_context(|_| ErrorKind::RepliClientError)?;
        let url = self.session.url.trim_end_matches('/');
        let url = format!(
            "{}{}/{}{}/{}{}",
            url,
            ENDPOINT_CLUSTER,
            cluster,
            ENDPOINT_CLUSTER_ACTION,
            action,
            ENDPOINT_CLUSTER_ACTION_APPROVE,
        );
        debug!(self.logger, "About to POST action approve request"; "url" => &url);
        let response = Response::perform(client.post(&url))?;
        response.check_status()?;
        Ok(())
    }

    /// Dispprove a PENDING_APPROVE action so it will not be scheduled.
    pub fn action_disapprove(&self, cluster: &str, action: Uuid) -> Result<()> {
        let client = ReqwestClient::builder()
            .build()
            .with_context(|_| ErrorKind::RepliClientError)?;
        let url = self.session.url.trim_end_matches('/');
        let url = format!(
            "{}{}/{}{}/{}{}",
            url,
            ENDPOINT_CLUSTER,
            cluster,
            ENDPOINT_CLUSTER_ACTION,
            action,
            ENDPOINT_CLUSTER_ACTION_DISAPPROVE,
        );
        debug!(self.logger, "About to POST action disapprove request"; "url" => &url);
        let response = Response::perform(client.post(&url))?;
        response.check_status()?;
        Ok(())
    }

    /// Send an `ApplyObject` to Replicate Core to request changes.
    pub fn apply(&self, object: ApplyObject) -> Result<Value> {
        let client = ReqwestClient::builder()
            .build()
            .with_context(|_| ErrorKind::RepliClientError)?;
        let url = self.session.url.trim_end_matches('/');
        let url = format!("{}{}", url, ENDPOINT_APPLY);
        debug!(self.logger, "About to POST apply request"; "url" => &url);
        let response = Response::perform(client.post(&url).json(&object))?;

        // Check apply-specific errors.
        if response.status.as_u16() == 400 {
            let remote: ErrorsCollection = response.body_as()?;
            return Err(ErrorKind::ApplyValidation(remote).into());
        }
        response.check_status()?;

        // Decode and return response payload on success.
        let remote = response.body;
        debug!(
            self.logger,
            "Recevied success response from apply API";
            "response" => ?remote
        );
        Ok(remote)
    }

    /// Schedule a refresh task for the given cluster.
    pub fn cluster_refresh(&self, cluster: &str) -> Result<()> {
        let client = ReqwestClient::builder()
            .build()
            .with_context(|_| ErrorKind::RepliClientError)?;
        let url = self.session.url.trim_end_matches('/');
        let url = format!(
            "{}{}/{}{}",
            url, ENDPOINT_CLUSTER, cluster, ENDPOINT_CLUSTER_REFRESH
        );
        debug!(self.logger, "About to POST cluster refresh request"; "url" => &url);
        let response = Response::perform(client.post(&url))?;
        response.check_status()?;
        Ok(())
    }

    /// Instantiate a new Replicante API client from CLI arguments.
    pub fn from_cli<'b>(cli: &ArgMatches<'b>, logger: &'a Logger) -> Result<RepliClient<'a>> {
        let sessions = SessionStore::load(cli)?;
        let name = sessions.active_name(cli);
        let session = match sessions.active(cli) {
            Some(session) => session,
            None => return Err(ErrorKind::SessionNotFound(name).into()),
        };
        info!(logger, "SSO session from CLI"; "session" => name, "instance" => &session.url);
        Ok(RepliClient::new(session, logger))
    }

    /// Instantiate a new Replicante API client with the given session.
    pub fn new(session: Session, logger: &'a Logger) -> RepliClient<'a> {
        RepliClient { logger, session }
    }
}

/// Response instance to store status code and JSON body.
struct Response {
    body: serde_json::Value,
    status: StatusCode,
}

impl Response {
    /// Perform the request and store needed response attributes.
    pub fn perform(request: reqwest::blocking::RequestBuilder) -> Result<Self> {
        let response = request
            .send()
            .with_context(|_| ErrorKind::RepliClientError)?;
        let status = response.status();
        let body = response
            .json()
            .with_context(|_| ErrorKind::RepliClientDecode)?;
        Ok(Response { body, status })
    }

    /// Decode the body of the response to the given type.
    pub fn body_as<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let body = self.body.clone();
        let body = serde_json::from_value(body).with_context(|_| ErrorKind::RepliClientDecode)?;
        Ok(body)
    }

    /// Check the HTTP response status code for common errors.
    pub fn check_status(&self) -> Result<()> {
        match self.status {
            // Missing resources or authentication errors.
            StatusCode::NOT_FOUND => Err(ErrorKind::RepliClientNotFound.into()),

            // Status < 400 indicate success of the operation.
            status if status.as_u16() < 400 => Ok(()),

            // Other remote errors.
            _ => {
                let body = self.body.clone();
                let remote: SerializableFail =
                    serde_json::from_value(body).with_context(|_| ErrorKind::RepliClientDecode)?;
                let mut error: Option<failure::Error> = None;
                for layer in remote.layers.into_iter() {
                    let layer = format!("(remote) {}", layer);
                    let err = match error {
                        None => failure::err_msg(layer),
                        Some(error) => error.context(failure::err_msg(layer)).into(),
                    };
                    error = Some(err);
                }
                match error {
                    None => Err(ErrorKind::RepliClientRemote.into()),
                    Some(error) => {
                        let error = error.context(ErrorKind::RepliClientRemote);
                        Err(error.into())
                    }
                }
            }
        }
    }
}
