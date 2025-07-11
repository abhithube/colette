use colette_util::{CryptoError, base64_decode, base64_encode};
use serde::{Deserialize, Serialize};

pub const PAGINATION_LIMIT: usize = 24;

pub trait Cursor {
    type Data: Serialize + for<'de> Deserialize<'de>;

    fn to_cursor(&self) -> Self::Data;
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub cursor: Option<String>,
}

pub struct Paginator;

impl Paginator {
    pub fn decode_cursor<T>(raw: &str) -> Result<T, CursorError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let decoded = base64_decode(raw)?;
        let data = serde_json::from_slice::<T>(&decoded)?;

        Ok(data)
    }

    pub fn encode_cursor<T>(data: &T) -> Result<String, CursorError>
    where
        T: Serialize,
    {
        let serialized = serde_json::to_vec(data)?;

        Ok(base64_encode(&serialized))
    }

    pub fn paginate<T>(mut items: Vec<T>, limit: usize) -> Result<Paginated<T>, CursorError>
    where
        T: Cursor,
    {
        let mut cursor: Option<String> = None;

        if items.len() > limit {
            items = items.into_iter().take(limit).collect();
            if let Some(last) = items.last() {
                cursor = Some(Self::encode_cursor(&last.to_cursor())?);
            }
        }

        Ok(Paginated { items, cursor })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CursorError {
    #[error(transparent)]
    Base64(#[from] CryptoError),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("cannot be empty")]
    Empty,
}
