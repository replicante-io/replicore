//! Configuration options for Platform HTTP(S) clients.
use std::time::Duration;

use anyhow::Result;
use reqwest::Client;
use reqwest::ClientBuilder;

/// Options to initialise clients with.
pub struct ClientOptions {
    /// Address of the API server to connect to, with trailing slash.
    pub address: String,

    /// Timeout for requests made by the client.
    pub timeout: Duration,

    /// Timeout for new connections initialised by the client.
    pub timeout_connect: Duration,

    /// Certificates Authority bundle for TLS verification.
    pub tls_ca_bundle: Option<String>,
    // TODO: tls_client_key
}

impl ClientOptions {
    /// Return a [`ClientBuilder`] configured with these options.
    pub fn client(&self, user_agent: &str) -> Result<ClientBuilder> {
        let mut builder = Client::builder()
            .connect_timeout(self.timeout_connect)
            .timeout(self.timeout)
            .user_agent(user_agent);

        // Configure additional CA certificates.
        if let Some(ca_bundle) = &self.tls_ca_bundle {
            let certs = reqwest::Certificate::from_pem_bundle(ca_bundle.as_bytes())?;
            for cert in certs {
                builder = builder.add_root_certificate(cert);
            }
        }

        Ok(builder)
    }

    /// Define options for API clients.
    pub fn url<S>(address: S) -> ClientOptionsBuilder
    where
        S: Into<String>,
    {
        ClientOptionsBuilder {
            address: address.into(),
            timeout: Duration::from_secs(30),
            timeout_connect: Duration::from_secs(1),
            tls_ca_bundle: None,
        }
    }
}

/// Incrementally build [`ClientOptions`] objects.`
pub struct ClientOptionsBuilder {
    address: String,
    timeout: Duration,
    timeout_connect: Duration,
    tls_ca_bundle: Option<String>,
}

impl ClientOptionsBuilder {
    /// Use the provided CA bundle, in PEM format.
    pub fn ca_bundle<S>(&mut self, bundle: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.tls_ca_bundle = Some(bundle.into());
        self
    }

    /// All options are set, get a usable options object.
    pub fn client(self) -> ClientOptions {
        self.into()
    }
}

impl From<ClientOptionsBuilder> for ClientOptions {
    fn from(value: ClientOptionsBuilder) -> Self {
        let mut address = value.address;
        if !address.ends_with('/') {
            address.push('/');
        }
        ClientOptions {
            address,
            timeout: value.timeout,
            timeout_connect: value.timeout_connect,
            tls_ca_bundle: value.tls_ca_bundle,
        }
    }
}
