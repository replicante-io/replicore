use super::action_record;
use crate::args::Args;

#[test]
fn args_decode() {
    let record = action_record();
    let args = Args::decode(&record).expect("args should decode");
    assert_eq!(args.remote.url, "https://test.local:1234");
}

#[test]
fn args_decode_nothing() {
    let mut record = action_record();
    record.args = serde_json::json!(null);
    let error = match Args::decode(&record) {
        Ok(_) => panic!("decoding did not fail"),
        Err(error) => error,
    };
    assert!(error.is::<crate::errors::InvalidRecord>());
}

#[test]
fn args_decode_partial() {
    let mut record = action_record();
    record.args = serde_json::json!({
        "argument": "some",
        "remote": {},
    });
    let error = match Args::decode(&record) {
        Ok(_) => panic!("decoding did not fail"),
        Err(error) => error,
    };
    assert!(error.is::<crate::errors::InvalidRecord>());
}

#[test]
fn args_timeout_not_set() {
    let mut record = action_record();
    record.args = serde_json::json!({
        "remote": {
            "url": "test",
        },
    });
    let args = Args::decode(&record).expect("args decoding failed unexpectedly");
    assert_eq!(args.remote.timeout, None);
}

#[test]
fn args_timeout_set_to_null() {
    let mut record = action_record();
    record.args = serde_json::json!({
        "remote": {
            "url": "test",
            "timeout": null,
        },
    });
    let args = Args::decode(&record).expect("args decoding failed unexpectedly");
    assert_eq!(args.remote.timeout, Some(None));
}

#[test]
fn args_timeout_set_to_seconds() {
    let mut record = action_record();
    record.args = serde_json::json!({
        "remote": {
            "url": "test",
            "timeout": 120,
        },
    });
    let args = Args::decode(&record).expect("args decoding failed unexpectedly");
    assert_eq!(args.remote.timeout, Some(Some(120)));
}
