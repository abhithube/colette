pub use api_key_repository::*;
use chrono::{DateTime, Utc};
pub use create_api_key_handler::*;
pub use delete_api_key_handler::*;
pub use get_api_key_handler::*;
pub use list_api_keys_handler::*;
pub use update_api_key_handler::*;
use uuid::Uuid;
pub use validate_api_key_handler::*;

use crate::pagination::Cursor;

mod api_key_repository;
mod create_api_key_handler;
mod delete_api_key_handler;
mod get_api_key_handler;
mod list_api_keys_handler;
mod update_api_key_handler;
mod validate_api_key_handler;

#[derive(Debug, Clone)]
pub struct ApiKey {
    pub id: Uuid,
    pub title: String,
    pub preview: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ApiKeyCursor {
    pub created_at: DateTime<Utc>,
}

impl Cursor for ApiKey {
    type Data = ApiKeyCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            created_at: self.created_at,
        }
    }
}
