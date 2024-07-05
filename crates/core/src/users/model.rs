use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
