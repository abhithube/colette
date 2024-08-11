pub const PAGINATION_LIMIT: usize = 24;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub cursor: Option<String>,
}
