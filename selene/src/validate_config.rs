use std::path::{Path, PathBuf};

use selene_lib::CheckerConfig;
use serde::Serialize;

use crate::standard_library::StandardLibraryError;

#[derive(Debug, Serialize)]
pub struct InvalidConfigError {
    error: String,
    source: PathBuf,
    location: Option<Location>,
}

#[derive(Debug, Serialize)]
pub struct Location {
    line: usize,
}

impl InvalidConfigError {
    pub fn rich_output(self) -> String {
        todo!("rich output for InvalidConfigError")
    }
}

pub fn validate_config(config_path: &Path) -> Result<(), InvalidConfigError> {
    let config_contents = match std::fs::read_to_string(config_path) {
        Ok(config_contents) => config_contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(());
        }
        Err(error) => {
            return Err(InvalidConfigError {
                error: error.to_string(),
                source: config_path.to_path_buf(),
                location: None,
            });
        }
    };

    let config = match toml::from_str::<CheckerConfig<toml::Value>>(&config_contents) {
        Ok(config) => config,
        Err(error) => {
            let location = error.line_col().map(|(line, ..)| Location { line });

            return Err(InvalidConfigError {
                error: error.to_string(),
                source: config_path.to_path_buf(),
                location,
            });
        }
    };

    match crate::standard_library::collect_standard_library(
        &config,
        config.std(),
        &std::env::current_dir().unwrap(),
        &None,
    ) {
        Ok(_) => Ok(()),

        Err(StandardLibraryError::BaseStd { source, .. }) => Err(InvalidConfigError {
            error: source.to_string(),
            source: config_path.to_path_buf(),
            location: None,
        }),

        Err(StandardLibraryError::Io { source, path }) => Err(InvalidConfigError {
            error: source.to_string(),
            source: path,
            location: None,
        }),

        Err(StandardLibraryError::Roblox(report)) => Err(InvalidConfigError {
            error: report.to_string(),
            source: config_path.to_path_buf(),
            location: config_contents
                .lines()
                .enumerate()
                .find_map(|(line, line_contents)| {
                    // good enough
                    if line_contents.contains("roblox") {
                        Some(Location { line: line + 1 })
                    } else {
                        None
                    }
                }),
        }),

        Err(StandardLibraryError::Toml { source, path }) => Err(InvalidConfigError {
            error: source.to_string(),
            source: path,
            location: source.line_col().map(|(line, ..)| Location { line }),
        }),

        Err(StandardLibraryError::Yml { source, path }) => Err(InvalidConfigError {
            error: source.to_string(),
            source: path,
            location: source.location().map(|location| Location {
                line: location.line(),
            }),
        }),
    }
}
