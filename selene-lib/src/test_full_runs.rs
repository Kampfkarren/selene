use crate::{
    test_util::{test_full_run, test_full_run_config, test_full_run_config_with_output},
    CheckerConfig,
};

#[test]
fn function_overriding() {
    test_full_run("function_overriding", "function_overriding");
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

// Plugins
#[test]
fn plugin_block_table_calls() {
    test_full_run("plugins/block_table_calls", "block_table_calls");
}

#[test]
fn plugin_incomplete_function_calls() {
    test_full_run(
        "plugins/incomplete_function_calls",
        "incomplete_function_calls",
    );
}
