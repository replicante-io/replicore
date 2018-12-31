use std::sync::Arc;

use failure::ResultExt;
use zookeeper::ZkError;

use super::super::super::super::ErrorKind;
use super::super::super::super::NodeId;
use super::super::super::super::Result;
use super::super::super::super::admin::Election;
use super::super::super::ElectionAdminBehaviour;
use super::super::ElectionInfo;
use super::super::ElectionCandidateInfo;
use super::super::client::Client;
use super::super::constants::PREFIX_ELECTION;


/// Iterate over zookeeper-backed elections
pub struct ZooKeeperElections {
    client: Arc<Client>,
    elections: Option<Vec<String>>,
}

impl ZooKeeperElections {
    pub fn new(client: Arc<Client>) -> ZooKeeperElections {
        ZooKeeperElections {
            client,
            elections: None,
        }
    }
}

impl ZooKeeperElections {
    /// Load the list of elections currently in Zookeeper.
    fn load_elections(&mut self) -> Result<()> {
        let keeper = self.client.get()?;
        let elections = Client::get_children(&keeper, PREFIX_ELECTION, false)
            .context(ErrorKind::Backend("elections listing"))?;
        let elections = elections.iter().map(|election| {
            format!("{}/{}", PREFIX_ELECTION, election)
        }).rev().collect();
        self.elections = Some(elections);
        Ok(())
    }
}

impl Iterator for ZooKeeperElections {
    type Item = Result<Election>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.elections.is_none() {
            if let Err(error) = self.load_elections() {
                // Cache an empty list to avoid endlessly attempting load after error.
                self.elections = Some(Vec::new());
                return Some(Err(error));
            }
        }

        // Look for still existing elections and decode their info.
        let elections = self.elections.as_mut().expect("ZooKeeperElections::elections to be set");
        while let Some(election) = elections.pop() {
            match ZooKeeperElectionAdmin::from_path(Arc::clone(&self.client), &election) {
                Err(error) => return Some(Err(error)),
                Ok(Some(election)) => return Some(Ok(election)),
                Ok(None) => continue,
            };
        }

        // There is nothing more to do.
        None
    }
}


/// Zookeeper specifics for election administration
pub struct ZooKeeperElectionAdmin {
    client: Arc<Client>,
    path: String,
}

impl ZooKeeperElectionAdmin {
    /// Model an election from name.
    pub fn from_name(client: Arc<Client>, name: &str) -> Result<Option<Election>> {
        let id = Client::hash_from_key(name);
        let path = format!("{}/{}", PREFIX_ELECTION, id);
        ZooKeeperElectionAdmin::from_path(client, &path)
    }

    /// Model an election rooted at the given path.
    pub fn from_path(client: Arc<Client>, path: &str) -> Result<Option<Election>> {
        let keeper = client.get()?;
        let info = match Client::get_data(&keeper, &path, false) {
            Ok((info, _)) => info,
            Err(ZkError::NoNode) => return Ok(None),
            Err(error) => {
                let error = Err(error).context(ErrorKind::Backend("election info fetch"));
                return error.map_err(|e| e.into());
            }
        };
        let info: ElectionInfo = match serde_json::from_slice(&info) {
            Ok(info) => info,
            Err(error) => {
                let error = Err(error).context(ErrorKind::Decode("election information"));
                return error.map_err(|e| e.into());
            }
        };
        let behaviour = Box::new(ZooKeeperElectionAdmin {
            client,
            path: path.to_string(),
        });
        return Ok(Some(Election::new(info.name, behaviour)));
    }
}

impl ZooKeeperElectionAdmin {
    fn candidates(&self) -> Result<Option<Vec<String>>> {
        let keeper = self.client.get()?;
        let candidates = match Client::get_children(&keeper, &self.path, false) {
            Ok(candidates) => candidates,
            Err(ZkError::NoNode) => return Ok(None),
            Err(error) => {
                let error = Err(error).context(ErrorKind::Backend("election candidates lookup"));
                return error.map_err(|e|e.into());
            }
        };
        Ok(Some(candidates))
    }
}

impl ElectionAdminBehaviour for ZooKeeperElectionAdmin {
    fn primary(&self) -> Result<Option<NodeId>> {
        let candidates = match self.candidates()? {
            None => return Ok(None),
            Some(candidates) => candidates,
        };
        let primary = match candidates.get(0) {
            None => return Ok(None),
            Some(primary) => primary,
        };
        let path = format!("{}/{}", self.path, primary);
        let keeper = self.client.get()?;
        let payload = match Client::get_data(&keeper, &path, false) {
            Ok((payload, _)) => payload,
            Err(ZkError::NoNode) => return Ok(None),
            Err(error) => {
                let error = Err(error).context(ErrorKind::Backend("election primary lookup"));
                return error.map_err(|e| e.into());
            }
        };
        let primary: ElectionCandidateInfo = match serde_json::from_slice(&payload) {
            Ok(primary) => primary,
            Err(error) => {
                let error = Err(error).context(ErrorKind::Decode("election candidate information"));
                return error.map_err(|e| e.into());
            }
        };
        Ok(Some(primary.owner))
    }

    fn secondaries_count(&self) -> Result<usize> {
        let count = match self.candidates()? {
            None => 0,
            Some(candidates) => candidates.len(),
        };
        // Ignore the primary, if any.
        let count = match count {
            0 => 0,
            1 => 0,
            n => n - 1,
        };
        Ok(count)
    }

    fn step_down(&self) -> Result<bool> {
        let candidates = match self.candidates()? {
            None => return Ok(false),
            Some(candidates) => candidates,
        };
        let primary = match candidates.get(0) {
            None => return Ok(false),
            Some(primary) => primary,
        };
        let path = format!("{}/{}", &self.path, primary);
        let keeper = self.client.get()?;
        Client::delete(&keeper, &path, None).context(ErrorKind::Backend("election step-down"))?;
        Ok(true)
    }
}
