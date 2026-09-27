#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{self:?}")]
    EventChannelClosed,

    #[error(transparent)]
    Compositor(#[from] CompositorError)
}

/// Error that directly relates to interfacing with Niri
#[derive(Debug, thiserror::Error)]
pub enum CompositorError {
    #[error("{self:?}")]
    FailedRequestCommunication(String),

    #[error("{self:?}")]
    ErrorResponse(String),

    #[error("{self:?}")]
    UnexpectedResponse(String),
}
