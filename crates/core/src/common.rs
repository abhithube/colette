use uuid::Uuid;

pub const PAGINATION_LIMIT: usize = 24;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Paginated<T> {
    pub has_more: bool,
    pub data: Vec<T>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct CursorPaginated<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct PaginationParams {
    pub limit: u64,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug)]
pub struct FindManyParams {
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct FindOneParams {
    pub id: Uuid,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub user_id: Uuid,
    pub profile_id: Uuid,
}
