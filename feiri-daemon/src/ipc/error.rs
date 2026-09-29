use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{self:?}")]
    Handler(#[from] crate::handlers::Error),

    #[error("{self:?}")]
    ServerAddrInUse { addr: PathBuf },

    #[error("{self:?}")]
    UnexpectedSocketFileConflict {
        file_path: PathBuf,
        reason: &'static str,
    },

    #[error("{self:?}")]
    FailedToOpenLockFile(#[source] std::io::Error),

    #[error("{self:?}")]
    FailedToAcquireLock(#[source] std::fs::TryLockError),

    #[error("{self:?}")]
    FailedToBindSocket(#[source] std::io::Error),

    #[error("{self:?}")]
    FailedToReadSocketMetadata(#[source] std::io::Error),

    #[error("{self:?}")]
    FailedToRemoveStaleSocket(#[source] std::io::Error),

    #[error("{self:?}")]
    FailedToAcceptIncomingConnection(#[source] std::io::Error),

    #[error("{self:?}")]
    FailedToReadStreamRequest(#[source] std::io::Error),

    #[error("{self:?}")]
    FailedToSerializeRequest(#[source] serde_json::Error),

    #[error("{self:?}")]
    UnexpectedIpcServerFailure(String),
}
