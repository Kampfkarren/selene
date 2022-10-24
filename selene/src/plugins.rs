use std::{fmt::Display, path::PathBuf};

use color_eyre::eyre::Context;
use selene_lib::CheckerConfig;
use serde::{Deserialize, Serialize};

use crate::opts::Options;

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct PluginAuthorization {
    path: PathBuf,
    allowed: bool,
}

fn plugin_authorization_path() -> PathBuf {
    let data_local_dir = dirs::data_local_dir().expect("could not find data local directory");
    data_local_dir
        .join("selene")
        .join("plugin_authorization.yml")
}

fn plugin_authorizations() -> color_eyre::Result<Vec<PluginAuthorization>> {
    let path = plugin_authorization_path();
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(error).with_context(|| format!("error when reading {}", path.display()))
        }
    };

    Ok(serde_yaml::from_str(&contents)?)
}

pub fn authorize_plugins<V>(
    options: &Options,
    config: &mut CheckerConfig<V>,
    canon_filename: PathBuf,
) {
    if options.allow_plugins {
        return;
    }

    let (mut authorizations, error) = match plugin_authorizations() {
        Ok(authorizations) => {
            if let Some(authorization) = authorizations
                .iter()
                .find(|authorization| authorization.path == canon_filename)
            {
                if authorization.allowed {
                    return;
                }

                config.plugins.clear();
                return;
            } else {
                (authorizations, None)
            }
        }

        Err(error) => (Vec::new(), Some(error.to_string())),
    };

    if !atty::is(atty::Stream::Stdin) {
        todo!("implement non-interactive mode");
    }

    let mut prompt = format!(
        "{} wants to load {} plugins.",
        match canon_filename.parent() {
            Some(parent) => parent.display(),
            None => canon_filename.display(),
        },
        config.plugins.len()
    );

    if let Some(error) = error {
        prompt.push_str(&format!(
            "\nwhen trying to see if it had permission, the following error occurred:\n{error}",
        ));
    }

    prompt.push_str("\ndo you want to authorize the use of the plugins?");

    match inquire::Select::new(
        &prompt,
        vec![
            PluginAuthorizationChoice::Yes,
            PluginAuthorizationChoice::Never,
            PluginAuthorizationChoice::No,
        ],
    )
    .prompt()
    {
        Ok(PluginAuthorizationChoice::Yes) => {
            authorizations.push(PluginAuthorization {
                path: canon_filename,
                allowed: true,
            });
        }

        Ok(PluginAuthorizationChoice::Never) => {
            config.plugins.clear();

            authorizations.push(PluginAuthorization {
                path: canon_filename,
                allowed: false,
            });
        }

        Ok(PluginAuthorizationChoice::No) => {
            config.plugins.clear();
            return;
        }

        // We shouldn't have gotten this far, but maybe it knows something atty doesn't
        Err(inquire::InquireError::NotTTY) => {
            todo!("implement non-interactive mode (caught by inquire)");
        }

        Err(error) => {
            eprintln!("error when prompting for the use of plugins: {error}");
            std::process::exit(1);
        }
    }

    let path = plugin_authorization_path();
    if let Err(error) = std::fs::create_dir_all(path.parent().unwrap()) {
        eprintln!("error when creating directory for plugin authorization file: {error}");
        std::process::exit(1);
    }

    if let Err(error) = std::fs::write(&path, serde_yaml::to_string(&authorizations).unwrap()) {
        eprintln!("error when writing plugin authorization file: {error}");
        std::process::exit(1);
    }
}

#[derive(Clone, Copy)]
enum PluginAuthorizationChoice {
    Yes,
    No,
    Never,
}

impl Display for PluginAuthorizationChoice {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                Self::Yes => "yes",
                Self::No => "not this time",
                Self::Never => "no, never",
            }
        )
    }
}
