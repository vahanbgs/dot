use std::fs;

use directories_next::BaseDirs;
use serde::Deserialize;

/// How this machine behaves, as against `Manifest`, which is what the tracked
/// tree says about itself. This one is not versioned with anything.
#[derive(Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub auto_deploy: bool,
}

pub fn load(base_dirs: &BaseDirs) -> anyhow::Result<Config> {
    let path = base_dirs.config_dir().join("dot/config.toml");

    let config_content = match fs::read_to_string(path) {
        Ok(s) => s,
        _ => return Ok(Config::default()),
    };

    Ok(toml::from_str(&config_content)?)
}
