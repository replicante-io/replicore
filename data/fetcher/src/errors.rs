error_chain! {
    links {
        Client(::replicante_agent_client::Error, ::replicante_agent_client::ErrorKind);
    }
}
