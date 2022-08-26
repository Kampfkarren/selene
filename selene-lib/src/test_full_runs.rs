use crate::{test_util::test_full_run_config, CheckerConfig};

#[test]
fn function_overriding() {
    test_full_run_config(
        "function_overriding",
        "function_overriding",
        CheckerConfig::default(),
    );
}
