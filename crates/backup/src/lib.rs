use bytes::Bytes;
use dyn_clone::DynClone;

pub mod netscape;
pub mod opml;

pub trait BackupManager: Send + Sync + DynClone + 'static {
    type Data;

    fn import(&self, raw: Bytes) -> Result<Self::Data, Error>;

    fn export(&self, data: Self::Data) -> Result<Bytes, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to serialize backup data")]
    Serialize,

    #[error("failed to deserialize backup data")]
    Deserialize,
}

dyn_clone::clone_trait_object!(<T> BackupManager<Data = T>);
