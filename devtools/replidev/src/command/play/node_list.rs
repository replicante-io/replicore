use prettytable::row;

use crate::platform::node_list::list_nodes;
use crate::Conf;
use crate::Result;

/// List running node pods.
///
/// Example output:
///   NODE                CLUSTER    STORE PORT  CLIENT PORT  AGENT PORT  STATUS   POD ID
///   play-node-rS3KQZOw  mongo-rs   10000       10000        10001       Running  206f19a3692f
///   play-node-Niu57N4O  zookeeper  10100       10101        10102       Stopped  817215a1fb8f
///   play-node-B6ZM7FWZ  postgres   10200       10200        -           Running  e72f080534c8
pub async fn run(conf: &Conf) -> Result<i32> {
    let nodes = list_nodes(conf).await?;
    let mut table = prettytable::Table::new();
    table.add_row(row![
        "NODE",
        "CLUSTER",
        "STORE PORT",
        "CLIENT PORT",
        "AGENT PORT",
        "STATUS",
        "POD ID",
    ]);
    for node in nodes {
        let pod_id = &node.id[0..12];
        let port_agent = node.port_agent.as_deref().unwrap_or("-");
        let port_client = node.port_client.as_ref().unwrap_or(&node.port_store);
        table.add_row(row![
            node.node,
            node.cluster,
            node.port_store,
            port_client,
            port_agent,
            node.status,
            pod_id,
        ]);
    }

    let format = prettytable::format::FormatBuilder::new()
        .column_separator(' ')
        .padding(0, 2)
        .build();
    table.set_format(format);
    table.printstd();
    Ok(0)
}
