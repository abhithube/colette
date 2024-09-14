use bytes::{Buf, Bytes};
use colette_opml::Opml;

use crate::BackupManager;

#[derive(Default)]
pub struct OpmlManager;

impl BackupManager for OpmlManager {
    type T = Opml;

    fn import(&self, raw: Bytes) -> Result<Self::T, crate::Error> {
        quick_xml::de::from_reader(raw.reader()).map_err(|_| crate::Error::Deserialize)
    }

    fn export(&self, data: Self::T) -> Result<Bytes, crate::Error> {
        let raw = quick_xml::se::to_string_with_root("opml", &data)
            .map_err(|_| crate::Error::Serialize)?;

        Ok(format!(r#"<?xml version="1.0" encoding="UTF-8"?>{}"#, raw).into())
    }
}
