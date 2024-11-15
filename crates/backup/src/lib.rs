use bytes::Bytes;
use dyn_clone::DynClone;

pub mod netscape;
pub mod opml;

pub trait BackupManager: Send + Sync + DynClone {
    type T;

    fn import(&self, raw: Bytes) -> Result<Self::T, Error>;

    fn export(&self, data: Self::T) -> Result<Bytes, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to serialize backup data")]
    Serialize,

    #[error("failed to deserialize backup data")]
    Deserialize,
}
