use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Paginated<T> {
    pub has_more: bool,
    pub data: Vec<T>,
}

pub struct FindOneParams<'a> {
    pub id: &'a str,
    pub profile_id: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub user_id: String,
    pub profile_id: String,
}
