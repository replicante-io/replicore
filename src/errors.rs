error_chain! {
    foreign_links {
        YamlDecode(::serde_yaml::Error);
    }
}
