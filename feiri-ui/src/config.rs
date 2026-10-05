use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub font: FontConfig
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowConfig {
    pub border_radius: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font: FontConfig {
                family: "Noto Sans".into(),
                size: 14,
            },
        }
    }
}
