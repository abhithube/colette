use core::str;

use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};

use crate::DataEncoder;

#[derive(Debug, Clone)]
pub struct Base64Encoder;

impl<T: Serialize + for<'a> Deserialize<'a>> DataEncoder<T> for Base64Encoder {
    fn encode(&self, data: &T) -> Result<String, anyhow::Error> {
        let raw = serde_json::to_string(data)?;
        let encoded = general_purpose::STANDARD_NO_PAD.encode(raw);

        Ok(encoded)
    }

    fn decode(&self, raw: &str) -> Result<T, anyhow::Error> {
        let decoded = general_purpose::STANDARD_NO_PAD.decode(raw)?;
        let data_str = str::from_utf8(&decoded)?;

        let data = serde_json::from_str::<T>(data_str)?;

        Ok(data)
    }
}
