#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Fatal UI platform error: {0}")]
    Platform(#[from] slint::PlatformError),

    #[error(transparent)]
    EventLoopError(#[from] slint::EventLoopError),

    #[error("{self:?}")]
    FailedIpcCommunication(#[from] feiri_ipc::socket::Error),

    #[error("{self:?}")]
    FeiriErrorRequest(String),

    #[error("{self:?}")]
    FailedToHandleEvent(String),

    #[error("{self:?}")]
    FailedToReadEvent(String),

    #[error("{self:?}")]
    UnexpectedIpcResponse {
        expected: feiri_ipc::Response,
        received: feiri_ipc::Response,
    },

    #[error("The UI window has been dropped and is no longer available")]
    UIDropped,

    #[error("{self:?}")]
    FailedToConvertIndexToSlot,

    //config specific
    #[error(transparent)]
    Config(#[from] feiri_core::config::ConfigError),

    #[error("{self:?}")]
    InvalidUiConfigColor(String),
}
