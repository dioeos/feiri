#[derive(Debug, thiserror::Error)]
pub enum Error {
    //config specific
    #[error(transparent)]
    Config(#[from] feiri_core::config::ConfigError),
}
