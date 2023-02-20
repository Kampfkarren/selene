use std::{
    fs,
    io::BufReader,
    path::{Path, PathBuf},
};

use chrono::TimeZone;
use color_eyre::eyre::Context;
use selene_lib::{standard_library::StandardLibrary, CheckerConfig, RobloxStdSource};

use super::RobloxGenerator;

// Someday, this can be a global config
const HOUR_CACHE: i64 = 6;

pub fn collect_roblox_standard_library<V>(
    config: &CheckerConfig<V>,
    current_directory: &Path,
) -> color_eyre::Result<StandardLibrary> {
    let (cached_library, output_directory, output_location) = match config.roblox_std_source {
        RobloxStdSource::Floating => {
            let floating_file_directory = floating_file_directory()?;
            let floating_file = floating_file_directory.join("roblox.yml");

            if floating_file.exists() {
                let mut library: StandardLibrary = serde_yaml::from_reader(BufReader::new(
                    fs::File::open(&floating_file)
                        .context("Could not open the floating roblox standard library.")?,
                ))?;

                if (chrono::Local::now()
                    - chrono::Local
                        .timestamp_opt(library.last_updated.unwrap_or(0), 0)
                        .unwrap())
                .num_hours()
                    < HOUR_CACHE
                    && library.last_selene_version.as_deref() == Some(env!("CARGO_PKG_VERSION"))
                {
                    if let Some(base) = &library.base {
                        let base_library = StandardLibrary::from_name(base)
                            .expect("Roblox standard library had an invalid base");

                        library.extend(base_library);
                    }

                    return Ok(library);
                }

                (Some(library), floating_file_directory, floating_file)
            } else {
                (None, floating_file_directory, floating_file)
            }
        }

        // If we get to this stage, then it means the search for roblox.yml
        // already failed.
        RobloxStdSource::Pinned => (
            None,
            current_directory.to_owned(),
            current_directory.join("roblox.yml"),
        ),
    };

    let generated_std = RobloxGenerator::generate();

    match (generated_std, cached_library) {
        (Ok((contents, new_library)), _) => {
            fs::create_dir_all(&output_directory).with_context(|| {
                format!(
                    "Could not create the directory for the floating roblox standard library at {}",
                    output_directory.display()
                )
            })?;

            fs::write(&output_location, contents).with_context(|| {
                format!(
                    "Could not write the Roblox standard library to {}",
                    output_location.display()
                )
            })?;

            Ok(new_library)
        }

        (Err(error), Some(cached_library)) => {
            crate::error(&format!(
                "There was an error generating a new Roblox standard library: {error}\nUsing the cached one instead.",
            ));

            Ok(cached_library)
        }

        (Err(error), None) => Err(error),
    }
}

fn floating_file_directory() -> color_eyre::Result<PathBuf> {
    match dirs::cache_dir() {
        Some(cache_dir) => Ok(cache_dir.join("selene")),
        None => color_eyre::eyre::bail!("your platform is not supported"),
    }
}

pub fn update_roblox_std() -> color_eyre::Result<()> {
    let (contents, _) = RobloxGenerator::generate()?;

    let output_directory = floating_file_directory()?;
    let output_location = output_directory.join("roblox.yml");

    fs::create_dir_all(&output_directory).with_context(|| {
        format!(
            "Could not create the directory for the floating roblox standard library at {}",
            output_directory.display()
        )
    })?;

    fs::write(&output_location, contents).with_context(|| {
        format!(
            "Could not write the Roblox standard library to {}",
            output_location.display()
        )
    })?;

    Ok(())
}
