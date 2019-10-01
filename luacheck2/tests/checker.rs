use std::collections::HashMap;

use luacheck2::{Checker, CheckerErrorProblem};
use serde_json::json;

#[test]
fn can_create() {
    Checker::from_config::<serde_json::Value>(HashMap::new()).unwrap();
}

#[test]
fn errors_with_bad_config() {
    let mut config = HashMap::new();
    config.insert("empty_if".to_owned(), json!("oh no"));

    match Checker::from_config(config) {
        Err(error) => {
            assert_eq!(error.name, "empty_if");
            match error.problem {
                CheckerErrorProblem::ConfigDeserializeError(_) => {},
                other => panic!("error was not ConfigDeserializeError: {:?}", other),
            }
        },

        _ => panic!("from_config returned Ok"),
    }
}
