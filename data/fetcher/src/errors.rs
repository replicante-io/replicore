error_chain! {
    links {
        Client(::replicante_agent_client::Error, ::replicante_agent_client::ErrorKind);
        Store(::replicante_data_store::Error, ::replicante_data_store::ErrorKind);
    }
}
