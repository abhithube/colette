use core::str;
use std::str::Utf8Error;

use base64::{DecodeError, Engine, engine::general_purpose};
use serde::{Deserialize, Serialize};

pub fn encode<T: Serialize>(data: &T) -> Result<String, Error> {
    let raw = serde_json::to_string(data)?;
    let encoded = general_purpose::STANDARD_NO_PAD.encode(raw);

    Ok(encoded)
}

pub fn decode<T: for<'a> Deserialize<'a>>(raw: &str) -> Result<T, Error> {
    let decoded = general_purpose::STANDARD_NO_PAD.decode(raw)?;
    let data_str = str::from_utf8(&decoded)?;
    let data = serde_json::from_str::<T>(data_str)?;

    Ok(data)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Utf(#[from] Utf8Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Decode(#[from] DecodeError),
}
