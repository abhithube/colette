use colette_core::feeds::{FeedFindManyParams, FeedUpdateData};

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
    pub profile_id: &'a str,
    pub feed_id: i64,
}

#[derive(Debug)]
pub struct UpdateData<'a> {
    pub custom_title: Option<&'a str>,
}

impl<'a> From<&'a FeedUpdateData> for UpdateData<'a> {
    fn from(value: &'a FeedUpdateData) -> Self {
        Self {
            custom_title: value.custom_title.as_deref(),
        }
    }
}
