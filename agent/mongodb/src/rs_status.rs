use bson::Bson;
use bson::Document;

use unamed_agent::AgentError;
use unamed_agent::AgentResult;
use unamed_agent::models::ShardRole;


/// Extracts the lag (in seconds) from the primary.
pub fn lag(rs: &Document, last_op: i64) -> AgentResult<i64> {
    let primary = find_primary(rs)?;
    let head = extract_timestamp(primary)?;
    Ok(head - last_op)
}


/// Extracts the timestamp (in seconds) of the latest operation.
pub fn last_op(rs: &Document) -> AgentResult<i64> {
    let node = find_self(rs)?;
    extract_timestamp(node)
}


/// Extracts the Replica Set name from the output of replSetGetStatus.
pub fn name(rs: &Document) -> AgentResult<String> {
    let name = rs.get("set").ok_or(AgentError::ModelViolation(
        String::from("Unable to determine Replica Set name")
    ))?;
    if let &Bson::String(ref name) = name {
        Ok(name.clone())
    } else {
        Err(AgentError::ModelViolation(String::from(
            "Unexpeted Replica Set name type (should be String)"
        )))
    }
}


/// Extracts the node's role in the Replica Set.
pub fn role(rs: &Document) -> AgentResult<ShardRole> {
    let role = rs.get("myState").ok_or(AgentError::ModelViolation(
        String::from("Unable to determine Replica Set myState")
    ))?;
    if let &Bson::I32(state) = role {
        match state {
            0 => Ok(ShardRole::Unknown(String::from("STARTUP"))),
            1 => Ok(ShardRole::Primary),
            2 => Ok(ShardRole::Secondary),
            3 => Ok(ShardRole::Unknown(String::from("RECOVERING"))),
            5 => Ok(ShardRole::Unknown(String::from("STARTUP2"))),
            6 => Ok(ShardRole::Unknown(String::from("UNKNOWN"))),
            7 => Ok(ShardRole::Unknown(String::from("ARBITER"))),
            8 => Ok(ShardRole::Unknown(String::from("DOWN"))),
            9 => Ok(ShardRole::Unknown(String::from("ROLLBACK"))),
            10 => Ok(ShardRole::Unknown(String::from("REMOVED"))),
            _ => Err(AgentError::UnsupportedDatastore(
                String::from("Unkown MongoDB node state")
            ))
        }
    } else {
        Err(AgentError::ModelViolation(String::from(
            "Unexpeted Replica Set name type (should be I32)"
        )))
    }
}


/// Extract the value of rs.status().members[x].optime.ts.
fn extract_timestamp(member: &Document) -> AgentResult<i64> {
    let optime = member.get("optime").ok_or(AgentError::UnsupportedDatastore(
        String::from("Unable to determine node's optime")
    ))?;
    let timestamp = match optime {
        &Bson::Document(ref doc) => doc.get("ts").ok_or(
            AgentError::UnsupportedDatastore(
                String::from("Unable to determine node's optime timestamp")
            )
        ),
        _ => Err(AgentError::UnsupportedDatastore(
            String::from("Node's optime is not a document")
        ))
    }?;
    match timestamp {
        &Bson::TimeStamp(timestamp) => Ok((timestamp >> 32) as i64),
        _ => Err(AgentError::UnsupportedDatastore(
            String::from("Node's optime timestamp is not a timestamp")
        ))
    }
}


/// Iterates over RS members to find the first matching the predicate.
fn first_member<F: Fn(&Document) -> bool>(
    rs: &Document, condition: F
) -> AgentResult<Option<&Document>> {
    let members = rs.get("members").ok_or(AgentError::UnsupportedDatastore(
        String::from("Unable to find Replica Set members")
    ))?;
    let members = match members {
        &Bson::Array(ref array) => array,
        _ => return Err(AgentError::UnsupportedDatastore(String::from(
            "Unexpeted members list type (should be an Array)"
        )))
    };
    for member in members {
        if let &Bson::Document(ref member) = member {
            if condition(member) {
                return Ok(Some(member));
            }
        }
    }
    Ok(None)
}

/// Looks for the details of the primary member of the replica set.
fn find_primary(rs: &Document) -> AgentResult<&Document> {
    first_member(rs, |member| {
        match member.get("state") {
            Some(&Bson::I32(1)) => true,
            _ => false
        }
    })?.ok_or(AgentError::DatastoreError(
        String::from("Unable to find PRIMARY member")
    ))
}

/// Looks for the current member of the replica set.
fn find_self(rs: &Document) -> AgentResult<&Document> {
    first_member(rs, |member: &Document| -> bool {
        match member.get("self") {
            Some(&Bson::Boolean(true)) => true,
            _ => false
        }
    })?.ok_or(AgentError::UnsupportedDatastore(
        String::from("Unable to find self in members list")
    ))
}


#[cfg(test)]
mod tests {
    use bson::Bson;
    use bson::Document;

    fn make_rs() -> Document {
        doc! {
            "set": "test-rs",
            "members": [{
                "_id": 0,
                "optime": {
                    "ts": Bson::TimeStamp((1514677701 as i64) << 32)
                },
                "self": false,
                "state": 1
            }, {
                "_id": 1,
                "optime": {
                    "ts": Bson::TimeStamp((1514677698 as i64) << 32)
                },
                "self": true
            }]
        }
    }

    #[test]
    fn compute_lag() {
        let rs = make_rs();
        let lag = super::lag(&rs, 1514677698).unwrap();
        assert_eq!(lag, 3);
    }

    
    #[test]
    fn compute_last_op() {
        let rs = make_rs();
        let timestamp = super::last_op(&rs).unwrap();
        assert_eq!(timestamp, 1514677698);
    }

    mod test_extract_timestamp {
        use bson::Bson;
        use unamed_agent::AgentError;
        use super::super::extract_timestamp;

        #[test]
        fn finds_timestamp() {
            let member = doc! {
                "optime": {
                    "ts": Bson::TimeStamp((1514677701 as i64) << 32)
                }
            };
            let timestamp = extract_timestamp(&member).unwrap();
            assert_eq!(1514677701, timestamp);
        }

        #[test]
        fn no_optime() {
            let member = doc! {};
            let timestamp = extract_timestamp(&member);
            match timestamp {
                Err(AgentError::UnsupportedDatastore(ref msg)) => assert_eq!(
                    "Unable to determine node's optime", msg
                ),
                Err(err) => panic!("Unexpeted error: {:?}", err),
                Ok(success) => panic!("Unexpeted success: {:?}", success)
            };
        }

        #[test]
        fn optime_not_a_doc() {
            let member = doc! {
                "optime": 4
            };
            let timestamp = extract_timestamp(&member);
            match timestamp {
                Err(AgentError::UnsupportedDatastore(ref msg)) => assert_eq!(
                    "Node's optime is not a document", msg
                ),
                Err(err) => panic!("Unexpeted error: {:?}", err),
                Ok(success) => panic!("Unexpeted success: {:?}", success)
            };
        }

        #[test]
        fn optime_has_no_ts() {
            let member = doc! {
                "optime": {}
            };
            let timestamp = extract_timestamp(&member);
            match timestamp {
                Err(AgentError::UnsupportedDatastore(ref msg)) => assert_eq!(
                    "Unable to determine node's optime timestamp", msg
                ),
                Err(err) => panic!("Unexpeted error: {:?}", err),
                Ok(success) => panic!("Unexpeted success: {:?}", success)
            };
        }

        #[test]
        fn optime_ts_not_a_timestamp() {
            let member = doc! {
                "optime": {
                    "ts": 4
                }
            };
            let timestamp = extract_timestamp(&member);
            match timestamp {
                Err(AgentError::UnsupportedDatastore(ref msg)) => assert_eq!(
                    "Node's optime timestamp is not a timestamp", msg
                ),
                Err(err) => panic!("Unexpeted error: {:?}", err),
                Ok(success) => panic!("Unexpeted success: {:?}", success)
            };
        }
    }

    mod test_first_member {
        use super::make_rs;
        use super::super::first_member;
        use unamed_agent::AgentError;

        #[test]
        fn members_not_an_array() {
            let rs = doc! {"members": 2};
            let member = first_member(&rs, |_| true);
            assert_eq!(true, member.is_err());
            match member {
                Err(AgentError::UnsupportedDatastore(ref msg)) => assert_eq!(
                    "Unexpeted members list type (should be an Array)", msg
                ),
                Err(err) => panic!("Unexpected error: {:?}", err),
                Ok(success) => panic!("Unexpected success: {:?}", success)
            };
        }

        #[test]
        fn member_not_found() {
            let rs = make_rs();
            let member = first_member(&rs, |_| false).unwrap();
            match member {
                None => (),
                Some(ref m) => panic!("Found unexpected member: {:?}", m)
            };
        }

        #[test]
        fn no_members() {
            let rs = doc! {};
            let member = first_member(&rs, |_| true);
            assert_eq!(true, member.is_err());
            match member {
                Err(AgentError::UnsupportedDatastore(ref msg)) => {
                    assert_eq!("Unable to find Replica Set members", msg);
                },
                Err(err) => panic!("Unexpected error: {:?}", err),
                Ok(success) => panic!("Unexpected success: {:?}", success)
            };
        }

        #[test]
        fn returns_first() {
            let rs = make_rs();
            let member = first_member(&rs, |_| true).unwrap().unwrap();
            let id = member.get_i32("_id").unwrap();
            assert_eq!(id, 0);
        }

        mod test_find_primary {
            use super::super::make_rs;
            use super::super::super::find_primary;
            use unamed_agent::AgentError;

            #[test]
            fn found() {
                let rs = make_rs();
                let primary = find_primary(&rs).unwrap();
                let id = primary.get_i32("_id").unwrap();
                assert_eq!(id, 0);
            }

            #[test]
            fn not_found() {
                let rs = doc! {"members": [{"_id": 3}]};
                let primary = find_primary(&rs);
                match primary {
                    Err(AgentError::DatastoreError(ref msg)) => {
                        assert_eq!("Unable to find PRIMARY member", msg);
                    },
                    Err(err) => panic!("Unexpected error: {:?}", err),
                    Ok(success) => panic!("Unexpected success: {:?}", success)
                };
            }
        }

        mod test_find_self {
            use super::super::make_rs;
            use super::super::super::find_self;
            use unamed_agent::AgentError;

            #[test]
            fn found() {
                let rs = make_rs();
                let primary = find_self(&rs).unwrap();
                let id = primary.get_i32("_id").unwrap();
                assert_eq!(id, 1);
            }

            #[test]
            fn not_found() {
                let rs = doc! {"members": [{"_id": 3}]};
                let primary = find_self(&rs);
                match primary {
                    Err(AgentError::UnsupportedDatastore(ref msg)) => {
                        assert_eq!("Unable to find self in members list", msg);
                    },
                    Err(err) => panic!("Unexpected error: {:?}", err),
                    Ok(success) => panic!("Unexpected success: {:?}", success)
                };
            }
        }
    }

    mod test_name {
        use unamed_agent::AgentError;
        use super::make_rs;
        use super::super::name;

        #[test]
        fn found() {
            let rs = make_rs();
            let rs_name = name(&rs).unwrap();
            assert_eq!("test-rs", rs_name);
        }

        #[test]
        fn not_a_string() {
            let rs = doc! {"set": 3};
            let rs_name = name(&rs);
            match rs_name {
                Err(AgentError::ModelViolation(ref msg)) => assert_eq!(
                    "Unexpeted Replica Set name type (should be String)", msg
                ),
                Err(err) => panic!("Unexpected error: {:?}", err),
                Ok(success) => panic!("Unexpected success: {:?}", success)
            };
        }

        #[test]
        fn not_present() {
            let rs = doc! {};
            let rs_name = name(&rs);
            match rs_name {
                Err(AgentError::ModelViolation(ref msg)) => assert_eq!(
                    "Unable to determine Replica Set name", msg
                ),
                Err(err) => panic!("Unexpected error: {:?}", err),
                Ok(success) => panic!("Unexpected success: {:?}", success)
            };
        }
    }

    mod test_role {
        use unamed_agent::AgentError;
        use unamed_agent::models::ShardRole;
        use super::super::role;

        #[test]
        fn found() {
            let rs = doc! {"myState": 1};
            let rs_role = role(&rs).unwrap();
            assert_eq!(ShardRole::Primary, rs_role);
        }

        #[test]
        fn not_present() {
            let rs = doc! {};
            let rs_role = role(&rs);
            match rs_role {
                Err(AgentError::ModelViolation(ref msg)) => assert_eq!(
                    "Unable to determine Replica Set myState", msg
                ),
                Err(err) => panic!("Unexpeted error: {:?}", err),
                Ok(success) => panic!("Unexpeted success: {:?}", success)
            };
        }

        #[test]
        fn not_an_i32() {
            let rs = doc! {"myState": "wrong"};
            let rs_role = role(&rs);
            match rs_role {
                Err(AgentError::ModelViolation(ref msg)) => assert_eq!(
                    "Unexpeted Replica Set name type (should be I32)", msg
                ),
                Err(err) => panic!("Unexpeted error: {:?}", err),
                Ok(success) => panic!("Unexpeted success: {:?}", success)
            };
        }

        #[test]
        fn not_not_supported() {
            let rs = doc! {"myState": 22};
            let rs_role = role(&rs);
            match rs_role {
                Err(AgentError::UnsupportedDatastore(ref msg)) => assert_eq!(
                    "Unkown MongoDB node state", msg
                ),
                Err(err) => panic!("Unexpeted error: {:?}", err),
                Ok(success) => panic!("Unexpeted success: {:?}", success)
            };
        }
    }
}
