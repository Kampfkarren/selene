use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::Context;
use selene_lib::{
    standard_library::{v1, StandardLibrary},
    CheckerConfig,
};

pub fn collect_standard_library<V>(
    config: &CheckerConfig<V>,
    standard_library_name: &str,
    directory: &Path,
    config_directory: &Option<PathBuf>,
) -> color_eyre::Result<Option<StandardLibrary>> {
    let mut standard_library: Option<StandardLibrary> = None;

    for segment in standard_library_name.split('+') {
        let segment_library = match from_name(config, segment, directory, config_directory)? {
            Some(segment_library) => segment_library,
            None => {
                if cfg!(feature = "roblox") && segment == "roblox" {
                    collect_roblox_standard_library(config, directory)?
                } else {
                    color_eyre::eyre::bail!("Could not find the standard library `{segment}`")
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
) -> color_eyre::Result<Option<StandardLibrary>> {
    let mut library: Option<StandardLibrary> = None;

    let mut directories = vec![directory];
    if let Some(directory) = config_directory {
        directories.push(directory);
    };

    for directory in directories {
        let toml_file = directory.join(format!("{standard_library_name}.toml"));
        if toml_file.exists() {
            let content = fs::read_to_string(&toml_file)?;

            let v1_library: v1::StandardLibrary = toml::from_str(&content)
                .with_context(|| format!("failed to read {}", toml_file.display()))?;

            library = Some(v1_library.into());
            break;
        } else {
            let mut yaml_file = directory.join(format!("{standard_library_name}.yml"));
            if !yaml_file.exists() {
                yaml_file = directory.join(format!("{standard_library_name}.yaml"));
            }

            if yaml_file.exists() {
                let content = fs::read_to_string(&yaml_file)
                    .with_context(|| format!("failed to read {}", yaml_file.display()))?;
                library = Some(serde_yaml::from_str(&content)?);
                break;
            }
        }
    }

    match library {
        Some(mut library) => {
            if let Some(base_name) = &library.base {
                if let Some(base) =
                    collect_standard_library(config, base_name, directory, config_directory)
                        .with_context(|| {
                            format!("failed to collect base standard library `{base_name}`")
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
