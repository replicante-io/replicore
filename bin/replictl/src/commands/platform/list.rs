use anyhow::Result;
use prettytable::row;
use replisdk::core::models::platform::PlatformTransport;
use slog::Logger;

use crate::apiclient::RepliClient;
use crate::context::ContextStore;

/// Execute the selected command.
pub async fn execute(logger: &Logger, cli: &crate::Cli) -> Result<i32> {
    let context = ContextStore::active_context(logger, cli).await?;
    let ns = context.namespace(&cli.context)?;
    let client = RepliClient::new(logger, context).await?;

    let mut table = prettytable::Table::new();
    table.add_row(row!["NAME", "ACTIVE", "KIND"]);

    for platform in client.platform_list(&ns).await? {
        table.add_row(row![
            platform.name,
            if platform.active { "Yes" } else { "No" },
            transport_name(&platform.transport),
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

/// Map known transports to user names.
fn transport_name(transport: &PlatformTransport) -> &'static str {
    match transport {
        PlatformTransport::Http(_) => "http",
    }
}
