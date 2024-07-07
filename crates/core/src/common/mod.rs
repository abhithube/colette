pub const PAGINATION_LIMIT: usize = 24;

#[derive(Debug)]
pub struct Paginated<T> {
    pub has_more: bool,
    pub data: Vec<T>,
}

#[derive(Debug)]
pub struct FindOneParams {
    pub id: String,
    pub profile_id: String,
}

#[derive(Debug)]
pub struct Session {
    pub user_id: String,
    pub profile_id: String,
}
