use std::path::{Path, PathBuf};

use selene_lib::CheckerConfig;
use serde::Serialize;

use crate::standard_library::StandardLibraryError;

#[derive(Debug, Serialize)]
pub struct InvalidConfigError {
    error: String,
    source: PathBuf,
    range: Option<ErrorRange>,
}

#[derive(Debug, Serialize)]
pub struct ErrorRange {
    start: usize,
    end: usize,
}

impl InvalidConfigError {
    pub fn rich_output(self) -> String {
        todo!("rich output for InvalidConfigError")
    }
}

impl From<std::ops::Range<usize>> for ErrorRange {
    fn from(range: std::ops::Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<serde_yaml::Location> for ErrorRange {
    fn from(location: serde_yaml::Location) -> Self {
        Self {
            start: location.index(),
            end: location.index(),
        }
    }
}

fn range_of_roblox(config_contents: &str) -> Option<ErrorRange> {
    let mut offset = 0;

    for line in config_contents.lines() {
        if !line.contains("roblox") {
            offset += line.len() + 1;
            continue;
        }

        // this is stupid and ideally we would be ast aware (which toml crate VERY much allows)
        return Some(ErrorRange {
            start: offset,
            end: offset + line.len(),
        });
    }

    None
}

// TODO: Test
pub fn validate_config(
    config_path: &Path,
    config_contents: &str,
) -> Result<(), InvalidConfigError> {
    let config = match toml::from_str::<CheckerConfig<toml::Value>>(&config_contents) {
        Ok(config) => config,
        Err(error) => {
            return Err(InvalidConfigError {
                error: error.to_string(),
                source: config_path.to_path_buf(),
                range: error.span().map(Into::into),
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
            range: None,
        }),

        Err(StandardLibraryError::Io { source, path }) => Err(InvalidConfigError {
            error: source.to_string(),
            source: path,
            range: None,
        }),

        Err(StandardLibraryError::Roblox(report)) => Err(InvalidConfigError {
            error: report.to_string(),
            source: config_path.to_path_buf(),
            range: range_of_roblox(config_contents),
        }),

        Err(StandardLibraryError::Toml { source, path }) => Err(InvalidConfigError {
            error: source.to_string(),
            source: path,
            range: source.span().map(Into::into),
        }),

        Err(StandardLibraryError::Yml { source, path }) => Err(InvalidConfigError {
            error: source.to_string(),
            source: path,
            range: source.location().map(Into::into),
        }),
    }
}
