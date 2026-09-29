#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{self:?}")]
    Service(#[from] crate::services::Error),
}
