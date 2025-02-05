use core::str;

use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};

pub fn encode<T: Serialize>(data: &T) -> anyhow::Result<String> {
    let raw = serde_json::to_string(data)?;
    let encoded = general_purpose::STANDARD_NO_PAD.encode(raw);

    Ok(encoded)
}

pub fn decode<T: for<'a> Deserialize<'a>>(raw: &str) -> anyhow::Result<T> {
    let decoded = general_purpose::STANDARD_NO_PAD.decode(raw)?;
    let data_str = str::from_utf8(&decoded)?;

    let data = serde_json::from_str::<T>(data_str)?;

    Ok(data)
}
