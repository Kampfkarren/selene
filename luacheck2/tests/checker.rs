// FIXME: These tests rely on nothing but empty_if triggering on them
use std::collections::HashMap;

use luacheck2::*;

use full_moon::parse;
use serde_json::json;

macro_rules! map {
    {
        $(
            $key:expr => $value:expr,
        )*
    } => {{
        let mut map = HashMap::new();
        $(
            map.insert($key, $value);
        )*
        map
    }};
}

#[test]
fn can_create() {
    Checker::from_config::<serde_json::Value>(CheckerConfig::default()).unwrap();
}

#[test]
fn errors_with_bad_config() {
    match Checker::from_config(CheckerConfig {
        config: map! {
            "empty_if".to_owned() => json!("oh no"),
        },
        ..CheckerConfig::default()
    }) {
        Err(error) => {
            assert_eq!(error.name, "empty_if");
            match error.problem {
                CheckerErrorProblem::ConfigDeserializeError(_) => {}
                other => panic!("error was not ConfigDeserializeError: {:?}", other),
            }
        }

        _ => panic!("from_config returned Ok"),
    }
}

#[test]
fn uses_rule_variation_allow() {
    let checker = Checker::from_config::<serde_json::Value>(CheckerConfig {
        rules: map! {
            "empty_if".to_owned() => RuleVariation::Allow,
        },
        ..CheckerConfig::default()
    })
    .unwrap();

    assert!(checker
        .test_on(&parse("if true then return end").unwrap())
        .is_empty());
}
