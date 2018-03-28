error_chain! {
    links {
        Tracing(::replicante_util_tracing::Error, ::replicante_util_tracing::ErrorKind);
    }

    foreign_links {
        IoError(::std::io::Error);
        YamlDecode(::serde_yaml::Error);
    }
}
