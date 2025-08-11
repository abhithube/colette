use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::user::UserId;

#[derive(Debug, Clone)]
pub struct Account {
    pub id: Uuid,
    pub sub: String,
    pub provider: String,
    pub password_hash: Option<String>,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
