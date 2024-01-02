//! Format `Platform` related objects.
use anyhow::Result;

use replisdk::core::models::api::PlatformEntry;
use replisdk::core::models::platform::Platform;
use replisdk::core::models::platform::PlatformTransport;
use replisdk::core::models::platform::PlatformTransportHttp;

/// Format a list of [`PlatformEntry`] objects into a table.
#[derive(Default)]
pub struct PlatformList {
    table: comfy_table::Table,
}

impl PlatformList {
    pub fn new() -> PlatformList {
        let mut table = comfy_table::Table::new();
        table.set_header(vec!["NAME", "ACTIVE"]);
        PlatformList { table }
    }
}

impl crate::formatter::PlatformList for PlatformList {
    fn append(&mut self, entry: &PlatformEntry) -> Result<()> {
        let active = crate::utils::yes_or_no(entry.active);
        self.table
            .add_row(vec![entry.name.clone(), active.to_string()]);
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        println!("{}", self.table);
        Ok(())
    }
}

/// Format a [`Platform`] for users to inspect.
pub fn show(platform: &Platform) {
    let active = crate::utils::yes_or_no(platform.active);
    println!("Platform Name: {}", platform.name);
    println!("As part of Namespace: {}", platform.ns_id);
    println!("Active: {}", active);

    println!();
    println!("Discovery options:");
    println!("  Interval: {} seconds", platform.discovery.interval);

    println!();
    println!("Transport configuration:");
    match &platform.transport {
        PlatformTransport::Http(transport) => show_transport_http(transport),
    };
}

fn show_transport_http(transport: &PlatformTransportHttp) {
    let ca_bundle = crate::utils::set_or_not(&transport.tls_ca_bundle);
    let skip_verify = crate::utils::yes_or_no(transport.tls_insecure_skip_verify);
    println!("  Mode: http");
    println!("  Platform URL: {}", transport.base_url);
    println!("  Certificate Authorities Bundle: {}", ca_bundle);
    println!(
        "  TLS Skip Verification (this is insecure): {}",
        skip_verify
    );
}
