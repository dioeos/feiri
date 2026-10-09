use super::error::Error;
use serde::{Deserialize, Serialize};

pub const FEIRI_DAEMON_CONFIG_PATH: &str = "feiri/config.toml";

#[derive(Debug, Deserialize, Serialize)]
pub struct DaemonConfig {
    pub default_marks: Vec<DefaultMark>,
}

#[allow(clippy::derivable_impls)]
impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            default_marks: Vec::new(),
        }
    }
}

impl DaemonConfig {
    pub fn load() -> Result<Self, Error> {
        let config = feiri_core::config::load_config::<DaemonConfig>(FEIRI_DAEMON_CONFIG_PATH)?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultMark {
    pub slot: u8,
    pub app_id: String,
}
