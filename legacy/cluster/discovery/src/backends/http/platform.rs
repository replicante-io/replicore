use failure::ResultExt;
use replisdk::core::models::platform::PlatformTransportHttp;
use replisdk::platform::models::ClusterDiscovery;
use replisdk::platform::models::ClusterDiscoveryResponse;
use reqwest::blocking::Client;
use reqwest::Url;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use crate::metrics::DISCOVERY_ERRORS;
use crate::metrics::DISCOVERY_TOTAL;
use crate::ErrorKind;
use crate::Result;

/// Response expected from an HTTP discovery server.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
struct DiscoveryResponse {
    pub clusters: Vec<ClusterDiscovery>,
    pub cursor: Option<String>,
}

/// Platform discovery iterator over HTTP(S).
///
/// Calls to `Iter::next` will perform HTTP(S) requests against a Platform
/// server to fetch cluster discovery records to return to the user.
///
/// The server must implement the Platform API discovery endpoint.
///
/// ## Pagination
///
/// Pagination is not currently supported by the Platform API.
/// Response pagination will be handled transparently by this iterator if it is added.
pub struct Iter {
    buffer: Vec<ClusterDiscovery>,
    client: Client,
    stop_iterating: bool,
    url: Url,
}

impl Iter {
    pub fn new(transport: PlatformTransportHttp) -> Result<Iter> {
        // Initialise the HTTP client to make requests with.
        let mut client = Client::builder();
        if let Some(ca_cert) = transport.tls_ca_bundle {
            let cert = reqwest::Certificate::from_pem(ca_cert.as_bytes())
                .context(ErrorKind::HttpCertLoad)?;
            client = client.add_root_certificate(cert);
        }
        if transport.tls_insecure_skip_verify {
            client = client.danger_accept_invalid_certs(true);
        }
        let client = client.build().context(ErrorKind::HttpClient)?;

        // Build the request URL only once.
        let url = Url::parse(&transport.base_url)
            .context(ErrorKind::HttpUrlInvalid)?
            .join("/discover")
            .context(ErrorKind::HttpUrlInvalid)?;
        let iter = Iter {
            buffer: Vec::new(),
            client,
            stop_iterating: false,
            url,
        };
        Ok(iter)
    }

    /// Fill the cluster discoveries buffer, if needed.
    ///
    /// The buffer is only refilled if it empty and we are not done iterating.
    /// If the buffer is empty and we fetch an empty page we are done iterating
    /// and the buffer stays empty.
    fn fill_buffer(&mut self) -> Result<()> {
        if !self.buffer.is_empty() || self.stop_iterating {
            return Ok(());
        }

        // Request a new page.
        let request = self.client.get(self.url.clone());
        let response = request.send().context(ErrorKind::HttpRequest)?;
        let response = response
            .error_for_status()
            .context(ErrorKind::HttpRequest)?;
        let page: ClusterDiscoveryResponse = response.json().context(ErrorKind::HttpRequest)?;

        // Update internal state from page.
        self.buffer = page.clusters;
        self.buffer.reverse();

        // Pagination is not supported yet so stop iterating once the buffer is empty.
        self.stop_iterating = true;
        Ok(())
    }
}

impl Iterator for Iter {
    type Item = Result<ClusterDiscovery>;

    fn next(&mut self) -> Option<Self::Item> {
        // Terminate iterating once the platform has no more clusters (or we failed to get them)..
        if self.buffer.is_empty() && self.stop_iterating {
            return None;
        }

        // Refill the discoveries buffer (only if needed).
        // On error we report back what happened and end iterating.
        if let Err(error) = self.fill_buffer() {
            DISCOVERY_ERRORS.with_label_values(&["http"]).inc();
            self.stop_iterating = true;
            return Some(Err(error));
        }

        // Return the top of the buffered discovery (could still be nothing).
        let item = self.buffer.pop().map(Ok);
        if item.is_some() {
            DISCOVERY_TOTAL.with_label_values(&["http"]).inc();
        }
        item
    }
}
