use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

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

// TODO: Test
pub fn validate_config(
    config_path: &Path,
    config_contents: &str,
) -> Result<(), InvalidConfigError> {
    let config_path_absolute = match config_path.canonicalize() {
        Ok(path) => path,
        Err(_) => config_path.to_path_buf(),
    };

    let config = match toml::from_str::<CheckerConfig<toml::Value>>(config_contents) {
        Ok(config) => config,
        Err(error) => {
            return Err(InvalidConfigError {
                error: error.to_string(),
                source: config_path.to_path_buf(),
                range: error.span().map(Into::into),
            });
        }
    };

    let spanned_config = toml::from_str::<HashMap<toml::Spanned<String>, toml::Spanned<toml::Value>>>(config_contents).expect("we should always be able to deserialize into a table if we can deserialize into a CheckerConfig");

    let std_range = spanned_config.get_key_value("std").map(|(key, value)| {
        let start = key.span().start;
        let end = value.span().end;

        ErrorRange { start, end }
    });

    match crate::standard_library::collect_standard_library(
        &config,
        config.std(),
        &std::env::current_dir().unwrap(),
        &None,
    ) {
        Ok(_) => Ok(()),

        Err(StandardLibraryError::BaseStd { source, .. }) => Err(InvalidConfigError {
            error: source.to_string(),
            source: config_path_absolute,
            range: std_range,
        }),

        // TODO: This triggers for bad `base` too
        Err(error @ StandardLibraryError::NotFound { .. }) => Err(InvalidConfigError {
            error: error.to_string(),
            source: config_path_absolute,
            range: std_range,
        }),

        Err(StandardLibraryError::Io { source, path }) => Err(InvalidConfigError {
            error: source.to_string(),
            source: path,
            range: None,
        }),

        Err(StandardLibraryError::Roblox(report)) => Err(InvalidConfigError {
            error: report.to_string(),
            source: config_path_absolute,
            range: std_range,
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
