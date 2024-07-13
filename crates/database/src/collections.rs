use colette_core::feeds::FeedFindManyParams;

#[derive(Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: &'a str,
}

impl<'a> From<&'a FeedFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a FeedFindManyParams) -> Self {
        Self {
            profile_id: &value.profile_id,
        }
    }
}

#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: String,
    pub title: &'a str,
    pub is_default: bool,
    pub profile_id: &'a str,
}
