use colette_core::feeds::FeedFindManyParams;

#[derive(Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: &'a str,
}

#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: String,
    pub profile_id: &'a str,
    pub feed_id: i64,
}

impl<'a> From<&'a FeedFindManyParams<'_>> for SelectManyParams<'a> {
    fn from(value: &'a FeedFindManyParams<'_>) -> Self {
        Self {
            profile_id: value.profile_id,
        }
    }
}
