use std::fs::File;

use failure::ResultExt;
use serde_yaml;

use replicante_data_models::ClusterDiscovery;

use super::super::metrics::DISCOVERY_ERRORS;
use super::super::metrics::DISCOVERY_TOTAL;
use super::super::ErrorKind;
use super::super::Result;

/// Serialization format for file discovery.
pub type DiscoveryFile = Vec<ClusterDiscovery>;

/// Iterator over results of file discovery.
pub struct Iter {
    data: Option<DiscoveryFile>,
    path: String,
}

impl Iter {
    /// Creates an iterator that reads the given file.
    pub fn new<S>(path: S) -> Iter
    where
        S: Into<String>,
    {
        Iter {
            data: None,
            path: path.into(),
        }
    }

    /// Loads the content of the file into memory to iterate over it.
    fn load_content(&mut self) -> Result<()> {
        let file = File::open(&self.path).with_context(|_| ErrorKind::Io(self.path.clone()))?;
        let mut content: DiscoveryFile = serde_yaml::from_reader(file)
            .with_context(|_| ErrorKind::YamlFile(self.path.clone()))?;
        content.reverse();
        self.data = Some(content);
        Ok(())
    }
}

impl Iterator for Iter {
    type Item = Result<ClusterDiscovery>;
    fn next(&mut self) -> Option<Self::Item> {
        DISCOVERY_TOTAL.with_label_values(&["file"]).inc();
        let data: &mut DiscoveryFile = match self.data {
            Some(ref mut data) => data,
            None => {
                match self.load_content() {
                    Ok(()) => (),
                    Err(error) => {
                        DISCOVERY_ERRORS.with_label_values(&["file"]).inc();
                        self.data = Some(Vec::new());
                        return Some(Err(error));
                    }
                };
                self.data.as_mut().unwrap()
            }
        };
        data.pop().map(Ok)
    }
}

#[cfg(test)]
mod tests {
    use replicante_data_models::ClusterDiscovery;

    use super::super::super::ErrorKind;
    use super::super::tests::fixture_path;
    use super::Iter;

    #[test]
    fn file_not_found() {
        let mut iter = Iter::new("/some/file/that/does/not/exists");
        match iter.next() {
            None => panic!("Should have returned a Some"),
            Some(Ok(_)) => panic!("Should have returned and Err"),
            Some(Err(error)) => match error.kind() {
                &ErrorKind::Io(ref path) => assert_eq!(path, "/some/file/that/does/not/exists"),
                _ => panic!("Invalid error: {:?}", error),
            },
        };
        assert!(iter.next().is_none());
    }

    #[test]
    fn example_file() {
        let mut iter = Iter::new(fixture_path("file.example.yaml"));
        let next = iter.next().unwrap().unwrap();
        assert_eq!(
            next,
            ClusterDiscovery::new(
                "mongodb-rs",
                vec![
                    "http://node1:37017".into(),
                    "http://node2:37017".into(),
                    "http://node3:37017".into(),
                ]
            )
        );
        assert!(iter.next().is_none());
    }

    #[test]
    fn no_clusters() {
        let mut iter = Iter::new(fixture_path("tests/no.clusters.yaml"));
        assert!(iter.next().is_none());
    }

    #[test]
    fn two_clusters() {
        let mut iter = Iter::new(fixture_path("tests/two.clusters.yaml"));
        let next = iter.next().unwrap().unwrap();
        assert_eq!(
            next,
            ClusterDiscovery::new(
                "test1",
                vec![
                    "http://node1:port/".into(),
                    "http://node2:port/".into(),
                    "http://node3:port/".into(),
                ]
            )
        );
        let next = iter.next().unwrap().unwrap();
        assert_eq!(
            next,
            ClusterDiscovery::new(
                "test2",
                vec!["http://node1:port/".into(), "http://node3:port/".into()]
            )
        );
        assert!(iter.next().is_none());
    }
}
