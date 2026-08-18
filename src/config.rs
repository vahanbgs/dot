use std::fs;

use directories_next::BaseDirs;
use serde::Deserialize;

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
