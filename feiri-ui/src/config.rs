use std::{
    env::{self, var_os},
    fs,
    path::{Path, PathBuf},
};

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
        }
    }
}

fn desktop_icon_name(path: &Path) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;

    let mut in_desktop_entry = false;
    for line in contents.lines().map(str::trim) {
        if line.starts_with('[') {
            in_desktop_entry = line == "[Desktop Entry]";
        } else if in_desktop_entry
            && let Some(icon) = line.strip_prefix("Icon=")
            && !icon.is_empty()
        {
            return Some(icon.to_owned());
        }
    }
    None
}

pub fn icon_for_app_id(app_id: &str) -> Option<slint::Image> {
    if app_id.is_empty() {
        return None;
    }

    let filename = format!("{app_id}.desktop");

    let mut dirs = Vec::new();

    if let Some(home) = var_os("XDG_DATA_HOME").filter(|var| !var.is_empty()) {
        dirs.push(home.into());
    } else if let Some(home) = var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share"));
    }

    let sys_dirs = var_os("XDG_DATA_DIRS")
        .filter(|var| !var.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into());

    dirs.extend(env::split_paths(&sys_dirs));

    let desktop_file = dirs
        .into_iter()
        .map(|dir| dir.join("applications").join(&filename))
        .find(|path| path.is_file())?;

    let icon_name = desktop_icon_name(&desktop_file)?;

    let icon_path = if Path::new(&icon_name).is_absolute() {
        PathBuf::from(icon_name)
    } else {
        freedesktop_icons::lookup(&icon_name)
            .with_size(32)
            .with_scale(1)
            .with_theme("hicolor")
            .find()?
    };

    slint::Image::load_from_path(&icon_path).ok()
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
