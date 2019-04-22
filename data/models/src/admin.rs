/// Version information reported by external systems.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Version {
    pub tag: String,
    pub version: ::semver::Version,
}

impl Version {
    pub fn new<S>(tag: S, version: ::semver::Version) -> Version
    where
        S: Into<String>,
    {
        Version {
            tag: tag.into(),
            version,
        }
    }
}
