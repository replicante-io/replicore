error_chain! {
    links {
        Discovery(::replicante_agent_discovery::Error, ::replicante_agent_discovery::ErrorKind);
        Tracing(::replicante_util_tracing::Error, ::replicante_util_tracing::ErrorKind);
    }

    foreign_links {
        IoError(::std::io::Error);
        YamlDecode(::serde_yaml::Error);
    }
}
