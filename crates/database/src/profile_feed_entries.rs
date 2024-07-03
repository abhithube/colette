#[derive(Debug)]
pub struct InsertData<'a> {
    pub profile_feed_id: &'a str,
    pub feed_entry_id: i32,
}
