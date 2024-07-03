#[derive(Debug)]
pub struct InsertData<'a> {
    pub profile_id: &'a str,
    pub feed_id: i64,
}
