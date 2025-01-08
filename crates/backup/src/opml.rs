use std::io::{BufRead, Write};

use colette_opml::Opml;

use crate::BackupManager;

#[derive(Debug, Clone, Default)]
pub struct OpmlManager;

impl<R: BufRead> BackupManager<R> for OpmlManager {
    type Data = Opml;

    fn import(&self, reader: R) -> Result<Self::Data, crate::Error> {
        colette_opml::from_reader(reader).map_err(|_| crate::Error::Deserialize)
    }

    fn export(&self, writer: &mut dyn Write, data: Self::Data) -> Result<(), crate::Error> {
        colette_opml::to_writer(writer, data).map_err(|_| crate::Error::Serialize)
    }
}
