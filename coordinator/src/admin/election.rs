use super::super::NodeId;
use super::super::Result;
use super::super::backend::ElectionAdminBehaviour;


/// Admin tools for an election.
pub struct Election {
    behaviour: Box<dyn ElectionAdminBehaviour>,
    name: String,
}

impl Election {
    pub(crate) fn new(name: String, behaviour: Box<dyn ElectionAdminBehaviour>) -> Election {
        Election {
            behaviour,
            name
        }
    }
}

impl Election {
    /// Return the name for this election.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Fetch the `NodeId` of the primary for this election, if a primary is elected.
    pub fn primary(&self) -> Result<Option<NodeId>> {
        self.behaviour.primary()
    }

    /// The number of secondary nodes waiting to take over if needed.
    pub fn secondaries_count(&self) -> Result<usize> {
        self.behaviour.secondaries_count()
    }

    /// Strip the current primary of its role and forces a new election.
    pub fn step_down(&self) -> Result<bool> {
        self.behaviour.step_down()
    }
}


/// Iterator over elections.
pub struct Elections(Box<dyn Iterator<Item = Result<Election>>>);

impl Elections {
    pub fn new<I: Iterator<Item = Result<Election>> + 'static>(inner: I) -> Elections {
        Elections(Box::new(inner))
    }
}

impl Iterator for Elections {
    type Item = Result<Election>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
