use std::{
    fmt::Write,
    io::{BufReader, Read},
};

use colette_opml::Opml;

use crate::BackupManager;

#[derive(Default)]
pub struct OpmlManager;

impl BackupManager for OpmlManager {
    type T = Opml;

    fn import(&self, reader: Box<dyn Read>) -> Result<Self::T, crate::Error> {
        quick_xml::de::from_reader(BufReader::new(reader)).map_err(|_| crate::Error::Deserialize)
    }

    fn export(&self, writer: &mut dyn Write, data: Self::T) -> Result<(), crate::Error> {
        write!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)
            .map_err(|_| crate::Error::Serialize)?;

        quick_xml::se::to_writer_with_root(writer, "opml", &data)
            .map_err(|_| crate::Error::Serialize)
    }
}
