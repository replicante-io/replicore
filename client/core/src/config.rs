//! Configuration options for RepliCore Clients.
use std::time::Duration;

/// Options to initialise clients with.
pub struct ClientOptions {
    /// Address of the API server to connect to, with trailing slash.
    pub(crate) address: String,

    /// Timeout for requests made by the client.
    pub(crate) timeout: Duration,

    /// Timeout for new connections initialised by the client.
    pub(crate) timeout_connect: Duration,
    // TODO: tls_ca_bundle
    // TODO: tls_client_key
}

impl ClientOptions {
    /// Define options for API clients.
    pub fn url<S>(address: S) -> ClientOptionsBuilder
    where
        S: Into<String>,
    {
        ClientOptionsBuilder {
            address: address.into(),
            timeout: Duration::from_secs(5),
            timeout_connect: Duration::from_secs(1),
        }
    }
}

/// Incrementally build [`ClientOptions`] objects.`
pub struct ClientOptionsBuilder {
    address: String,
    timeout: Duration,
    timeout_connect: Duration,
}

impl ClientOptionsBuilder {
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
        }
    }
}
