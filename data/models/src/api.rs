/// Replicante version information.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Version {
    pub commit: String,
    pub taint: String,
    pub version: String,
}
