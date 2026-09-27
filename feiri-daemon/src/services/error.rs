
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{self:?}")]
    Niri(#[from] crate::niri::error::CompositorError)
}
