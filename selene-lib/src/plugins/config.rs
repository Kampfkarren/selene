use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct PluginConfig {
    pub source: PathBuf,
}
