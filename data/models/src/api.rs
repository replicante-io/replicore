/// Replicante version information.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Version {
    pub commit: String,
    pub taint: String,
    pub version: String,
}
