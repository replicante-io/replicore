use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde_yaml;

use super::super::Discovery;
use super::super::Result;


/// Serialization format for file discovery.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DiscoveryFile {
    pub cluster: String,
    pub targets: Vec<String>,
}


/// Iterator over results of file discovery.
pub struct Iter {
    cluster: String,
    targets: Vec<String>,
}

impl Iter {
    /// Creates an iterator over the loaded data.
    fn new(cluster: String, mut targets: Vec<String>) -> Iter {
        targets.reverse();
        Iter { cluster, targets }
    }

    /// Creates an iterator out of a YAML file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Iter> {
        Iter::from_yaml(File::open(path)?)
    }

    /// Creates an iterator out of a YAML stream.
    pub fn from_yaml<R: Read>(reader: R) -> Result<Iter> {
        let content: DiscoveryFile = serde_yaml::from_reader(reader)?;
        Ok(Iter::new(content.cluster, content.targets))
    }
}

impl Iterator for Iter {
    type Item = Result<Discovery>;
    fn next(&mut self) -> Option<Self::Item> {
        self.targets.pop().map(|target| {
            Ok(Discovery::new(self.cluster.clone(), target))
        })
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::Discovery;
    use super::Iter;

    #[test]
    fn empty() {
        let mut iter = Iter::new(String::from("cluster"), Vec::new());
        assert!(iter.next().is_none());
    }

    #[test]
    fn found() {
        let mut iter = Iter::new(String::from("cluster"), vec![
            String::from("A"), String::from("B"), String::from("C")
        ]);
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("cluster", "A"));
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("cluster", "B"));
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("cluster", "C"));
        assert!(iter.next().is_none());
    }

    #[test]
    fn from_yaml() {
        let cursor = Cursor::new(r#"cluster: c
targets:
    - a
    - b
    - c
"#);
        let mut iter = Iter::from_yaml(cursor).unwrap();
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("c", "a"));
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("c", "b"));
        assert_eq!(iter.next().unwrap().unwrap(), Discovery::new("c", "c"));
        assert!(iter.next().is_none());
    }
}
