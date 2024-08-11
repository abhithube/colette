use uuid::Uuid;

pub const PAGINATION_LIMIT: usize = 24;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub user_id: Uuid,
    pub profile_id: Uuid,
}
