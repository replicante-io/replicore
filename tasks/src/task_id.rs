use std::fmt;
use std::str::FromStr;

use replicante_util_rndid::RndId;

/// Task (probably) unique identifiers.
///
/// Use to distinguish each task from others and "connect" retires to the main task.
#[derive(Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct TaskId(RndId);

impl TaskId {
    pub fn new() -> TaskId {
        TaskId(RndId::new())
    }
}

impl FromStr for TaskId {
    type Err = ::failure::Error;
    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let id: RndId = s.parse()?;
        Ok(TaskId(id))
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::TaskId;

    #[test]
    fn ids_differ() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn from_string() {
        let raw_id = "ce84c2f150f72f1499d28b50c550c4c0";
        let id: TaskId = raw_id.parse().unwrap();
        assert_eq!(id.to_string(), raw_id);
    }

    #[test]
    fn from_string_upper() {
        let raw_id = "CE84c2f150f72f1499D28b50c550c4c0";
        let id: TaskId = raw_id.parse().unwrap();
        assert_eq!(id.to_string(), raw_id.to_lowercase());
    }

    #[test]
    #[should_panic(expected = "kind: Length")]
    fn from_string_invalid_length() {
        let raw_id = "ABC";
        let _id: TaskId = raw_id.parse().unwrap();
    }

    #[test]
    #[should_panic(expected = "DecodeError")]
    fn from_string_not_hex() {
        let raw_id = "%^84c2f150f72f1499d28b50c550c4c0";
        let _id: TaskId = raw_id.parse().unwrap();
    }
}
