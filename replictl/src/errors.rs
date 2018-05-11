error_chain! {
    links {
        Store(::replicante_data_store::Error, ::replicante_data_store::ErrorKind);
    }

    foreign_links {
        IoError(::std::io::Error);
        YamlDecode(::serde_yaml::Error);
    }
}
