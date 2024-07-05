#[derive(Debug)]
pub struct Paginated<T> {
    pub has_more: bool,
    pub data: Vec<T>,
}

#[derive(Debug)]
pub struct FindOneParams<'a> {
    pub id: &'a str,
    pub profile_id: &'a str,
}

#[derive(Debug)]
pub struct Session<'a> {
    pub user_id: &'a str,
    pub profile_id: &'a str,
}
