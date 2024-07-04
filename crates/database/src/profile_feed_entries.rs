#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: String,
    pub profile_feed_id: &'a str,
    pub feed_entry_id: i64,
}
