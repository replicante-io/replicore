/// Replicante version information.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Version {
    pub commit: &'static str,
    pub taint: &'static str,
    pub version: &'static str,
}
