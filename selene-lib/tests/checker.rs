use std::collections::HashMap;

use selene_lib::{standard_library::StandardLibrary, *};

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
    Checker::<serde_json::Value>::new(CheckerConfig::default(), StandardLibrary::default())
        .unwrap();
}

#[test]
fn errors_with_bad_config() {
    match Checker::new(
        CheckerConfig {
            config: map! {
                "empty_if".to_owned() => json!("oh no"),
            },
            ..CheckerConfig::default()
        },
        StandardLibrary::default(),
    ) {
        Err(error) => {
            assert_eq!(error.name, "empty_if");
            match error.problem {
                CheckerErrorProblem::ConfigDeserializeError(_) => {}
                other => panic!("error was not ConfigDeserializeError: {other:?}"),
            }
        }

        _ => panic!("new returned Ok"),
    }
}

#[test]
fn uses_lint_variation_allow() {
    let checker: Checker<serde_json::Value> = Checker::new(
        CheckerConfig {
            lints: map! {
                "empty_if".to_owned() => LintVariation::Allow,
            },
            ..CheckerConfig::default()
        },
        StandardLibrary::default(),
    )
    .unwrap();

    assert!(checker
        .test_on(&parse("if true then\n\treturn\nend").unwrap())
        .is_empty());
}
