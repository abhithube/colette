pub trait BackupManager: Send + Sync {
    type T;

    fn import(&self, raw: &str) -> Result<Self::T, Error>;

    fn export(&self, data: Self::T) -> Result<String, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to serialize backup data")]
    Serialize,

    #[error("failed to deserialize backup data")]
    Deserialize,
}
