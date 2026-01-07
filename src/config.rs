use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub auto_deploy: bool,
}
