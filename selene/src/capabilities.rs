//! Capabilities exist to abstract feature support from the version of selene,
//! making the code for the extension easier to read, as well as supporting debug
//! versions of selene before a version upgrade.
//!
//! Capabilities have a semver version, but these are only updated as needed
//! by official extensions. Pull requests to update capability versions where
//! useful for other purposes are welcome.

use crate::{json_output::JsonOutput, opts::DisplayStyle};

fn capabilities() -> serde_json::Value {
    serde_json::json!({
        "validateConfig": {
            "version": "1.0.0"
        }
    })
}

pub fn print_capabilities(display_style: DisplayStyle) {
    match display_style {
        DisplayStyle::Quiet | DisplayStyle::Rich => {
            println!("{}", serde_yaml::to_string(&capabilities()).unwrap());
        }

        // We can't print anything in legacy JSON mode, because common
        // extensions still read from it.
        DisplayStyle::Json => {}

        DisplayStyle::Json2 => {
            println!(
                "{}",
                serde_json::to_string(&JsonOutput::Capabilities(capabilities())).unwrap()
            );
        }
    }
}
