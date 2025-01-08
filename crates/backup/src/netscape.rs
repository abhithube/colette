use std::io::{Read, Write};

use colette_netscape::Netscape;

use crate::BackupManager;

#[derive(Debug, Clone, Default)]
pub struct NetscapeManager;

impl<R: Read> BackupManager<R> for NetscapeManager {
    type Data = Netscape;

    fn import(&self, reader: R) -> Result<Self::Data, crate::Error> {
        colette_netscape::from_reader(reader).map_err(|_| crate::Error::Deserialize)
    }

    fn export(&self, writer: &mut dyn Write, data: Self::Data) -> Result<(), crate::Error> {
        colette_netscape::to_writer(writer, data).map_err(|_| crate::Error::Serialize)
    }
}
