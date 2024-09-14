use std::{fmt::Write, io::Read};

pub mod opml;

pub trait BackupManager: Send + Sync {
    type T;

    fn import(&self, reader: Box<dyn Read>) -> Result<Self::T, Error>;

    fn export(&self, writer: &mut dyn Write, data: Self::T) -> Result<(), Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to serialize backup data")]
    Serialize,

    #[error("failed to deserialize backup data")]
    Deserialize,
}
