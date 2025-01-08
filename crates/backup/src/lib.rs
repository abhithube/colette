use std::io::{Read, Write};

pub mod netscape;
pub mod opml;

pub trait BackupManager<R: Read>: Send + Sync + 'static {
    type Data;

    fn import(&self, reader: R) -> Result<Self::Data, Error>;

    fn export(&self, writer: &mut dyn Write, data: Self::Data) -> Result<(), Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to serialize backup data")]
    Serialize,

    #[error("failed to deserialize backup data")]
    Deserialize,
}
