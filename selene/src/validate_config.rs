use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use selene_lib::CheckerConfig;
use serde::Serialize;

use crate::standard_library::StandardLibraryError;

#[derive(Debug, Serialize)]
pub struct InvalidConfigError {
    #[serde(serialize_with = "serialize_standard_library_error_to_string")]
    error: StandardLibraryError,
    source: PathBuf,
    range: Option<ErrorRange>,
}

fn serialize_standard_library_error_to_string<S: serde::Serializer>(
    error: &StandardLibraryError,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_str(&error.to_string())
}

#[derive(Debug, Serialize)]
pub struct ErrorRange {
    start: usize,
    end: usize,
}

impl InvalidConfigError {
    pub fn write_rich_output(
        &self,
        writer: &mut impl termcolor::WriteColor,
    ) -> std::io::Result<()> {
        let codespan_files = codespan_reporting::files::SimpleFile::new(
            self.source
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            std::fs::read_to_string(&self.source)?,
        );

        let mut diagnostic = codespan_reporting::diagnostic::Diagnostic::error()
            .with_message(self.error.to_string());

        if let Some(range) = &self.range {
            diagnostic =
                diagnostic.with_labels(vec![codespan_reporting::diagnostic::Label::primary(
                    (),
                    range.start..range.end,
                )]);
        }

        match codespan_reporting::term::emit(
            writer,
            &codespan_reporting::term::Config::default(),
            &codespan_files,
            &diagnostic,
        ) {
            Ok(_) => {}
            Err(codespan_reporting::files::Error::Io(io_error)) => return Err(io_error),
            Err(error) => unreachable!("unexpected codespan error: {error:#?}"),
        }

        Ok(())
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

pub fn validate_config(
    config_path: &Path,
    config_contents: &str,
    directory: &Path,
) -> Result<(), InvalidConfigError> {
    let config_path_absolute = match config_path.canonicalize() {
        Ok(path) => path,
        Err(_) => config_path.to_path_buf(),
    };

    let config = match toml::from_str::<CheckerConfig<toml::Value>>(config_contents) {
        Ok(config) => config,
        Err(error) => {
            return Err(InvalidConfigError {
                source: config_path.to_path_buf(),
                range: error.span().map(Into::into),
                error: StandardLibraryError::Toml {
                    source: error,
                    path: config_path.to_path_buf(),
                },
            });
        }
    };

    let spanned_config = toml::from_str::<HashMap<toml::Spanned<String>, toml::Spanned<toml::Value>>>(config_contents).expect("we should always be able to deserialize into a table if we can deserialize into a CheckerConfig");

    let std_range = spanned_config.get_key_value("std").map(|(key, value)| {
        let start = key.span().start;
        let end = value.span().end;

        ErrorRange { start, end }
    });

    let Err(error) = crate::standard_library::collect_standard_library(&config, config.std(), directory, &None) else {
        return Ok(());
    };

    match error {
        StandardLibraryError::BaseStd { .. } => Err(InvalidConfigError {
            error,
            source: config_path_absolute,
            range: std_range,
        }),

        StandardLibraryError::NotFound { .. } => Err(InvalidConfigError {
            source: config_path_absolute,
            range: std_range,
            error,
        }),

        StandardLibraryError::Io { ref path, .. } => Err(InvalidConfigError {
            source: path.clone(),
            range: None,
            error,
        }),

        StandardLibraryError::Roblox(..) => Err(InvalidConfigError {
            source: config_path_absolute,
            range: std_range,
            error,
        }),

        StandardLibraryError::Toml {
            ref source,
            ref path,
        } => Err(InvalidConfigError {
            source: path.clone(),
            range: source.span().map(Into::into),
            error,
        }),

        StandardLibraryError::Yml {
            ref source,
            ref path,
        } => Err(InvalidConfigError {
            source: path.clone(),
            range: source.location().map(Into::into),
            error,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_config_tests() {
        let mut tests_pass = true;

        for validate_config_test in std::fs::read_dir("./tests/validate_config").unwrap() {
            let validate_config_test = validate_config_test.unwrap();

            let config_path = validate_config_test.path().join("selene.toml");
            let config_contents = std::fs::read_to_string(&config_path).unwrap();

            let Err(validate_result) =
                validate_config(&config_path, &config_contents, &validate_config_test.path())
                else {
                    tests_pass = false;

                    eprintln!(
                        "{} did not error",
                        validate_config_test.file_name().to_string_lossy()
                    );

                    continue;
                };

            let mut rich_output_buffer = termcolor::NoColor::new(Vec::new());
            validate_result
                .write_rich_output(&mut rich_output_buffer)
                .unwrap();
            let rich_output = String::from_utf8(rich_output_buffer.into_inner()).unwrap();

            let expected_rich_output =
                std::fs::read_to_string(validate_config_test.path().join("rich_output.txt"));

            if let Ok(expected_rich_output) = expected_rich_output {
                if rich_output != expected_rich_output {
                    tests_pass = false;

                    eprintln!(
                        "validate_config test failed: {}\n{}",
                        validate_config_test.file_name().to_string_lossy(),
                        pretty_assertions::StrComparison::new(&rich_output, &expected_rich_output)
                    );
                }
            } else {
                std::fs::write(
                    validate_config_test.path().join("rich_output.txt"),
                    rich_output,
                )
                .unwrap();
            }
        }

        assert!(
            tests_pass,
            "validate_config tests failed: rerun with --nocapture to see output"
        );
    }
}
