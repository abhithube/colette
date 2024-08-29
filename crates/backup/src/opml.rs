use bytes::{Buf, Bytes};
use opml::OPML;

use crate::BackupManager;

#[derive(Default)]
pub struct OpmlManager;

impl BackupManager for OpmlManager {
    type T = OPML;

    fn import(&self, raw: Bytes) -> Result<Self::T, crate::Error> {
        OPML::from_reader(&mut raw.reader()).map_err(|_| crate::Error::Deserialize)
    }

    fn export(&self, data: Self::T) -> Result<Bytes, crate::Error> {
        let raw = data.to_string().map_err(|_| crate::Error::Serialize)?;

        Ok(format!(r#"<?xml version="1.0" encoding="UTF-8"?>{}"#, raw).into())
    }
}
