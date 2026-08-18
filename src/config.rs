use std::{collections::HashMap, fs, path::PathBuf};

use directories_next::BaseDirs;
use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub auto_deploy: bool,

    /// What a Nuke document renders to where its name does not say, keyed by
    /// the deployed path relative to the home directory. `.gitconfig` and
    /// Ghostty's `config` are named rather than extended, so nothing but this
    /// table can name their target.
    #[serde(default)]
    pub targets: HashMap<PathBuf, String>,
}

pub fn load(base_dirs: &BaseDirs) -> anyhow::Result<Config> {
    let path = base_dirs.config_dir().join("dot/config.toml");

    let config_content = match fs::read_to_string(path) {
        Ok(s) => s,
        _ => return Ok(Config::default()),
    };

    Ok(toml::from_str(&config_content)?)
}
