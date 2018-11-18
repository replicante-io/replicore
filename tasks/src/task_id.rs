use std::fmt;
use std::str::FromStr;

use data_encoding::HEXLOWER_PERMISSIVE;
use rand::Rng;

use super::Result;
use super::TaskError;


/// Task (probably) unique identifiers.
///
/// Use to distinguish each task from others and "connect" retires to the main task.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct TaskId(String);

impl TaskId {
    /// Return a new, random TaskId.
    pub fn new() -> TaskId {
        let mut rng = rand::thread_rng();
        let id: [u8; 16] = rng.gen();
        TaskId(HEXLOWER_PERMISSIVE.encode(&id))
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

impl FromStr for TaskId {
    type Err = ::failure::Error;
    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match HEXLOWER_PERMISSIVE.decode_len(s.len()) {
            Ok(16) => {
                // Make sure the ID is actually valid and not just the correct length.
                let mut buf = [0; 16];
                let decode_error: Result<_> = HEXLOWER_PERMISSIVE
                    .decode_mut(s.as_bytes(), &mut buf)
                    .map_err(|e| e.error.into());
                decode_error?;
                // But still store it as a string.
                Ok(TaskId(String::from(s).to_lowercase()))
            },
            _ => return Err(TaskError::Msg("invalid ID length".into()).into()),
        }
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
    #[should_panic(expected = "invalid ID length")]
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
