use std::{env::var_os, fs, io::ErrorKind, path::PathBuf};

use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid configuration at {path}: {reason}")]
    Invalid { path: PathBuf, reason: String },

    #[error("{self:?}")]
    MissingEnvironmentVariable { variable: String },

    #[error("Failed to serialize default config: {0}")]
    SerializeConfig(#[source] toml::ser::Error),

    #[error("Failed to write default config: {0}")]
    FailedToWriteDefaultConfig(#[source] std::io::Error),

    #[error("{self:?}")]
    UnexpectedFailure(String),

    #[error("Failed to create config parent directory")]
    FailedDirectory(#[source] std::io::Error),
}

pub fn load_config<T>(config_path: &str) -> Result<T, ConfigError>
where
    T: Default + Serialize + DeserializeOwned,
{
    let xdg_config_os_string =
        var_os("XDG_CONFIG_HOME").ok_or_else(|| ConfigError::MissingEnvironmentVariable {
            variable: "XDG_CONFIG_HOME".into(),
        })?;

    let path = PathBuf::from(xdg_config_os_string).join(config_path);

    let config_contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            if let Some(parent) = path.parent() {
                fs::create_dir(parent).map_err(ConfigError::FailedDirectory)?;
            }

            let default_config = T::default();
            let contents =
                toml::to_string(&default_config).map_err(ConfigError::SerializeConfig)?;

            fs::write(&path, &contents).map_err(ConfigError::FailedToWriteDefaultConfig)?;
            contents
        }
        Err(err) => {
            return Err(ConfigError::UnexpectedFailure(err.to_string()));
        }
    };

    let config: T = toml::from_str(&config_contents).map_err(|err| ConfigError::Invalid {
        path,
        reason: err.to_string(),
    })?;

    Ok(config)
}
