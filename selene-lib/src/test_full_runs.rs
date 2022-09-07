use crate::{
    test_util::{test_full_run_config, test_full_run_config_with_output},
    CheckerConfig,
};

#[test]
fn function_overriding() {
    test_full_run_config(
        "function_overriding",
        "function_overriding",
        CheckerConfig::default(),
    );
}

#[test]
fn test_std_mistakes() {
    test_full_run_config_with_output(
        "std_mistakes",
        "std_mistakes",
        CheckerConfig::default(),
        if cfg!(feature = "roblox") {
            "stderr"
        } else {
            "noroblox.stderr"
        },
    );
}

#[test]
#[cfg(feature = "roblox")]
fn test_std_mistakes_roblox() {
    test_full_run_config("std_mistakes", "roblox_mistakes", CheckerConfig::default());
}
