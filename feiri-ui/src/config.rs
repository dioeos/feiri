use super::Error;
use serde::{Deserialize, Serialize};
use slint::Color;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub font: FontConfig,
    pub colors: ColorsConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub strong_weight: u32,
    pub small_length: u32,
    pub medium_length: u32
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
    pub text_primary: String
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font: FontConfig {
                family: "Inter".into(),
                strong_weight: 700,
                small_length: 14,
                medium_length: 14
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
            }
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
