use std::string::FromUtf8Error;

use base64::{engine::general_purpose, DecodeError, Engine};
use serde::{Deserialize, Serialize};

pub(crate) fn encode_cursor<T: Serialize>(cursor: &T) -> Result<String, CursorError> {
    let raw = serde_json::to_string(cursor)?;
    let encoded = general_purpose::STANDARD_NO_PAD.encode(raw);

    Ok(encoded)
}

pub(crate) fn decode_cursor<T: for<'a> Deserialize<'a>>(cursor: &str) -> Result<T, CursorError> {
    let decoded = general_purpose::STANDARD_NO_PAD.decode(cursor)?;
    let cursor_str = String::from_utf8(decoded)?;

    let cursor = serde_json::from_str::<T>(&cursor_str)?;

    Ok(cursor)
}

#[derive(Debug, thiserror::Error)]
pub enum CursorError {
    #[error(transparent)]
    Decode(#[from] DecodeError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),
}
