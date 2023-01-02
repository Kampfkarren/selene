use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use color_eyre::eyre::Context;
use selene_lib::CheckerConfig;
use serde::{Deserialize, Serialize};

use crate::{
    json_output,
    opts::{DisplayStyle, Options},
};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct PluginAuthorizations {
    authorizations: Vec<PluginAuthorization>,
    version: u32,
}

impl Default for PluginAuthorizations {
    fn default() -> Self {
        Self {
            version: 1,

            authorizations: Default::default(),
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct PluginAuthorization {
    path: PathBuf,
    allowed: bool,
}

pub fn plugin_authorization_path() -> PathBuf {
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

    // PLUGIN TODO: Check version, stop if it's too high
    Ok(serde_yaml::from_str::<PluginAuthorizations>(&contents)?.authorizations)
}

pub fn authorize_plugins_prompt<V>(
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

        Err(error) => (Vec::new(), Some(format!("{:#}", error))),
    };

    if !atty::is(atty::Stream::Stdin) {
        config.plugins.clear();
        non_interactive_message(options, canon_filename);
        return;
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
            config.plugins.clear();
            non_interactive_message(options, canon_filename);
            return;
        }

        Err(error) => {
            eprintln!("error when prompting for the use of plugins: {error}");
            std::process::exit(1);
        }
    }

    write_plugin_authorizations_or_die(authorizations);
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

pub fn allow_plugin_by_bool(path: &Path, allowed: bool) -> color_eyre::Result<()> {
    let canonical_path = path.canonicalize()?;

    let mut authorizations = plugin_authorizations()?;

    if let Some(authorization) = authorizations
        .iter_mut()
        .find(|authorization| authorization.path == canonical_path)
    {
        authorization.allowed = allowed;
    } else {
        authorizations.push(PluginAuthorization {
            path: canonical_path.to_path_buf(),
            allowed,
        });
    }

    write_plugin_authorizations_or_die(authorizations);

    Ok(())
}

fn write_plugin_authorizations_or_die(new_authorizations: Vec<PluginAuthorization>) {
    let path = plugin_authorization_path();
    if let Err(error) = std::fs::create_dir_all(path.parent().unwrap()) {
        eprintln!("error when creating directory for plugin authorization file: {error}");
        std::process::exit(1);
    }

    if let Err(error) = std::fs::write(
        &path,
        serde_yaml::to_string(&PluginAuthorizations {
            version: 1,
            authorizations: new_authorizations,
        })
        .unwrap(),
    ) {
        eprintln!("error when writing plugin authorization file: {error}");
        std::process::exit(1);
    }
}

fn non_interactive_message(options: &Options, canon_filename: PathBuf) {
    match options.display_style() {
        DisplayStyle::Rich => {
            eprintln!("plugins attempted to run, but do not have permission. run with --allow-plugins to allow plugins if you grant them permission.");
        }

        DisplayStyle::Json2 => {
            json_output::print_json(json_output::JsonOutput::PluginsNotLoaded {
                authorization_path: plugin_authorization_path(),
                canon_filename,
            });
        }

        DisplayStyle::Quiet | DisplayStyle::Json => {}
    }
}
