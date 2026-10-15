use iroh_docs::NamespaceId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("document {0} is not held")]
    NotHeld(NamespaceId),
    #[error("document {0} is still held")]
    StillHeld(NamespaceId),
    /// The blob store, the docs engine or device storage failed.
    #[error(transparent)]
    Backend(#[from] anyhow::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Converts the error types of the iroh crates, which share no common one.
pub trait IrohResultExt<T> {
    fn iroh(self) -> Result<T>;
}

impl<T, E: Into<anyhow::Error>> IrohResultExt<T> for std::result::Result<T, E> {
    fn iroh(self) -> Result<T> {
        self.map_err(|err| Error::Backend(err.into()))
    }
}
