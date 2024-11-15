use bytes::{Buf, Bytes};
use colette_opml::Opml;

use crate::BackupManager;

#[derive(Debug, Clone, Default)]
pub struct OpmlManager;

impl BackupManager for OpmlManager {
    type T = Opml;

    fn import(&self, raw: Bytes) -> Result<Self::T, crate::Error> {
        colette_opml::from_reader(raw.reader()).map_err(|_| crate::Error::Deserialize)
    }

    fn export(&self, data: Self::T) -> Result<Bytes, crate::Error> {
        let mut buffer: Vec<u8> = Vec::new();

        colette_opml::to_writer(&mut buffer, data).map_err(|_| crate::Error::Serialize)?;

        Ok(buffer.into())
    }
}

dyn_clone::clone_trait_object!(BackupManager<T = Opml>);
