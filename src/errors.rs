error_chain! {
    foreign_links {
        IoError(::std::io::Error);
        YamlDecode(::serde_yaml::Error);
    }
}
