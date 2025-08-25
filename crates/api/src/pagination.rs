use colette_util::{CryptoError, base64_decode, base64_encode};
use serde::{Deserialize, Serialize};

use crate::common::ApiError;

pub const PAGINATION_LIMIT: usize = 24;

/// Paginated list of results
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Paginated<T: utoipa::ToSchema> {
    /// Current set of results
    pub(crate) items: Vec<T>,
    /// Pagination cursor, only present if more results are available
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cursor: Option<String>,
}

impl<T, U, V> TryFrom<colette_handler::Paginated<U, V>> for Paginated<T>
where
    T: From<U> + utoipa::ToSchema,
    V: Serialize,
{
    type Error = ApiError;

    fn try_from(value: colette_handler::Paginated<U, V>) -> Result<Self, Self::Error> {
        Ok(Self {
            items: value.items.into_iter().map(T::from).collect(),
            cursor: value.cursor.map(|e| encode_cursor(&e)).transpose()?,
        })
    }
}

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

#[derive(Debug, thiserror::Error)]
pub enum CursorError {
    #[error(transparent)]
    Base64(#[from] CryptoError),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}
