use bytes::{Buf, Bytes};
use colette_netscape::Netscape;

use crate::BackupManager;

#[derive(Debug, Clone, Default)]
pub struct NetscapeManager;

impl BackupManager for NetscapeManager {
    type Data = Netscape;

    fn import(&self, raw: Bytes) -> Result<Self::Data, crate::Error> {
        colette_netscape::from_reader(raw.reader()).map_err(|_| crate::Error::Deserialize)
    }

    fn export(&self, data: Self::Data) -> Result<Bytes, crate::Error> {
        let mut buffer: Vec<u8> = Vec::new();

        colette_netscape::to_writer(&mut buffer, data).map_err(|_| crate::Error::Serialize)?;

        Ok(buffer.into())
    }
}
