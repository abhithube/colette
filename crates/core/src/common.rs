pub const PAGINATION_LIMIT: u64 = 24;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("cannot be empty")]
    Empty,
}
