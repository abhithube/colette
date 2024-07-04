#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: String,
    pub profile_id: &'a str,
    pub feed_id: i64,
}
