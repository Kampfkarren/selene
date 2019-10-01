use std::collections::HashMap;

use luacheck2::Checker;

#[test]
fn can_create() {
    Checker::from_config::<serde_json::Value>(HashMap::new()).unwrap();
}
