use std::{env::var_os, fs, io::ErrorKind, path::PathBuf};

use super::Error;
use serde::{Deserialize, Serialize};
use slint::Color;

pub const FEIRI_CONFIG_FILE_PATH: &str = "feiri/ui-config.toml";

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub font: FontConfig,
    pub colors: ColorsConfig,
    pub icons: IconsConfig,
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        let xdg_config_os_string =
            var_os("XDG_CONFIG_HOME").ok_or_else(|| Error::MissingEnvironmentVariable {
                variable: "XDG_CONFIG_HOME".into(),
            })?;

        let mut config_path = PathBuf::from(xdg_config_os_string);
        config_path.push(FEIRI_CONFIG_FILE_PATH);

        let config_contents = match fs::read_to_string(&config_path) {
            Ok(contents) => contents,
            Err(err) if err.kind() == ErrorKind::NotFound => {
                if let Some(parent) = config_path.parent() {
                    fs::create_dir_all(parent).map_err(Error::FailedToCreateDir)?;
                }

                let default_config = Config::default();
                let contents =
                    toml::to_string(&default_config).map_err(Error::FailedToSerializeConfig)?;

                fs::write(&config_path, &contents).map_err(Error::FailedToWriteConfig)?;

                contents
            }
            Err(err) => {
                return Err(Error::ConfigFailure(err.to_string()));
            }
        };

        let config: Config =
            toml::from_str(&config_contents).map_err(|err| Error::FailedToDeserializeConfig {
                config_path: format!("{}", config_path.display()),
                reason: err.to_string(),
            })?;

        Ok(config)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub strong_weight: u32,
    pub small_length: u32,
    pub medium_length: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColorsConfig {
    pub background: String,
    pub surface: String,
    pub border: String,
    pub accent: String,
    pub secondary_accent: String,
    pub selected: String,
    pub hover: String,
    pub text_primary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IconsConfig {
    pub icon_theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font: FontConfig {
                family: "Inter".into(),
                strong_weight: 700,
                small_length: 14,
                medium_length: 14,
            },
            colors: ColorsConfig {
                background: "#F5F5F7".into(),
                surface: "#FFFFFFEE".into(),
                border: "#F5F5F7".into(),
                accent: "#D6E9FF".into(),
                secondary_accent: "#EAF4FF".into(),
                selected: "#DCEEFF".into(),
                hover: "#F0F0F3".into(),
                text_primary: "#1D1D1F".into(),
            },
            icons: IconsConfig {
                icon_theme: "hicolor".into(),
            },
        }
    }
}

pub fn parse_color(hex: &str) -> Result<Color, Error> {
    let hex = hex
        .strip_prefix('#')
        .ok_or_else(|| Error::InvalidConfig("color must start with '#'".into()))?;

    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| Error::InvalidConfig("invalid red channel".into()))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| Error::InvalidConfig("invalid green channel".into()))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| Error::InvalidConfig("invalid blue channel".into()))?;

            Ok(Color::from_rgb_u8(r, g, b))
        }

        8 => {
            // Slint/CSS notation: #RRGGBBAA
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| Error::InvalidConfig("invalid red channel".into()))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| Error::InvalidConfig("invalid green channel".into()))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| Error::InvalidConfig("invalid blue channel".into()))?;
            let a = u8::from_str_radix(&hex[6..8], 16)
                .map_err(|_| Error::InvalidConfig("invalid alpha channel".into()))?;

            Ok(Color::from_argb_u8(a, r, g, b))
        }

        _ => Err(Error::InvalidConfig("expected #RRGGBBAA or #RRGGBB".into())),
    }
}
