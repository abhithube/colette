use uuid::Uuid;

pub const PAGINATION_LIMIT: u64 = 24;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct IdParams {
    pub id: Uuid,
    pub user_id: Uuid,
}

impl IdParams {
    pub fn new(id: Uuid, user_id: Uuid) -> Self {
        Self { id, user_id }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("cannot be empty")]
    Empty,
}
