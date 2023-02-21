use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use selene_lib::{
    standard_library::{v1, StandardLibrary},
    CheckerConfig,
};

#[derive(Debug)]
pub enum StandardLibraryError {
    BaseStd {
        source: Box<StandardLibraryError>,
        name: String,
    },

    Io {
        source: std::io::Error,
        path: PathBuf,
    },

    NotFound {
        name: String,
    },

    Roblox(color_eyre::eyre::Report),

    Toml {
        source: toml::de::Error,
        path: PathBuf,
    },

    Yml {
        source: serde_yaml::Error,
        path: PathBuf,
    },
}

impl Display for StandardLibraryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StandardLibraryError::BaseStd { name, .. } => {
                write!(
                    formatter,
                    "failed to collect base standard library `{name}`",
                )
            }

            StandardLibraryError::Io { source, path } => {
                write!(
                    formatter,
                    "failed to read file `{}`: {source}",
                    path.display(),
                )
            }

            StandardLibraryError::NotFound { name } => {
                write!(formatter, "failed to find standard library: {name}")
            }

            StandardLibraryError::Roblox(report) => {
                write!(
                    formatter,
                    "failed to collect roblox standard library: {report}",
                )
            }

            StandardLibraryError::Toml { source, path } => {
                write!(
                    formatter,
                    "failed to parse toml file `{}`: {}",
                    path.display(),
                    source.message(),
                )
            }

            StandardLibraryError::Yml { source, path } => {
                write!(
                    formatter,
                    "failed to parse yml file `{}`: {source}",
                    path.display(),
                )
            }
        }
    }
}

pub fn collect_standard_library<V>(
    config: &CheckerConfig<V>,
    standard_library_name: &str,
    directory: &Path,
    config_directory: &Option<PathBuf>,
) -> Result<Option<StandardLibrary>, StandardLibraryError> {
    let mut standard_library: Option<StandardLibrary> = None;

    for segment in standard_library_name.split('+') {
        let segment_library = match from_name(config, segment, directory, config_directory)? {
            Some(segment_library) => segment_library,
            None => {
                if cfg!(feature = "roblox") && segment == "roblox" {
                    collect_roblox_standard_library(config, directory)
                        .map_err(StandardLibraryError::Roblox)?
                } else {
                    return Err(StandardLibraryError::NotFound {
                        name: segment.to_owned(),
                    });
                }
            }
        };

        match standard_library.as_mut() {
            Some(standard_library) => {
                standard_library.extend(segment_library);
            }

            None => {
                standard_library = Some(segment_library);
            }
        }
    }

    Ok(standard_library)
}

#[cfg(feature = "roblox")]
fn collect_roblox_standard_library<V>(
    config: &CheckerConfig<V>,
    directory: &Path,
) -> color_eyre::Result<StandardLibrary> {
    crate::roblox::collect_roblox_standard_library(config, directory)
}

#[cfg(not(feature = "roblox"))]
fn collect_roblox_standard_library<V>(
    _config: &CheckerConfig<V>,
    _directory: &Path,
) -> color_eyre::Result<StandardLibrary> {
    unreachable!()
}

fn from_name<V>(
    config: &CheckerConfig<V>,
    standard_library_name: &str,
    directory: &Path,
    config_directory: &Option<PathBuf>,
) -> Result<Option<StandardLibrary>, StandardLibraryError> {
    let mut library: Option<StandardLibrary> = None;

    let mut directories = vec![directory];
    if let Some(directory) = config_directory {
        directories.push(directory);
    };

    for directory in directories {
        let toml_file = directory.join(format!("{standard_library_name}.toml"));
        if toml_file.exists() {
            let content =
                fs::read_to_string(&toml_file).map_err(|error| StandardLibraryError::Io {
                    source: error,
                    path: toml_file.clone(),
                })?;

            let v1_library: v1::StandardLibrary =
                toml::from_str(&content).map_err(|error| StandardLibraryError::Toml {
                    source: error,
                    path: toml_file.clone(),
                })?;

            library = Some(v1_library.into());
            break;
        } else {
            let mut yaml_file = directory.join(format!("{standard_library_name}.yml"));
            if !yaml_file.exists() {
                yaml_file = directory.join(format!("{standard_library_name}.yaml"));
            }

            if yaml_file.exists() {
                let content =
                    fs::read_to_string(&yaml_file).map_err(|error| StandardLibraryError::Io {
                        source: error,
                        path: yaml_file.clone(),
                    })?;

                library = Some(serde_yaml::from_str(&content).map_err(|error| {
                    StandardLibraryError::Yml {
                        source: error,
                        path: yaml_file.clone(),
                    }
                })?);

                break;
            }
        }
    }

    match library {
        Some(mut library) => {
            if let Some(base_name) = &library.base {
                if let Some(base) =
                    collect_standard_library(config, base_name, directory, config_directory)
                        .map_err(|error| StandardLibraryError::BaseStd {
                            source: Box::new(error),
                            name: base_name.clone(),
                        })?
                {
                    library.extend(base);
                }
            }

            Ok(Some(library))
        }
        None => Ok(StandardLibrary::from_name(standard_library_name)),
    }
}
