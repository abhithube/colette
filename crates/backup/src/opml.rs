use std::io::{BufReader, Read, Write};

use colette_opml::Opml;

use crate::BackupManager;

#[derive(Default)]
pub struct OpmlManager;

impl BackupManager for OpmlManager {
    type T = Opml;

    fn import(&self, reader: Box<dyn Read>) -> Result<Self::T, crate::Error> {
        colette_opml::from_reader(BufReader::new(reader)).map_err(|_| crate::Error::Deserialize)
    }

    fn export(&self, writer: &mut dyn Write, data: Self::T) -> Result<(), crate::Error> {
        colette_opml::to_writer(writer, data).map_err(|_| crate::Error::Serialize)
    }
}
