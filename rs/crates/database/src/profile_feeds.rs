use colette_core::feeds::FeedFindManyParams;
use uuid::Uuid;

#[derive(Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: &'a Uuid,
}

impl<'a> From<&'a FeedFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a FeedFindManyParams) -> Self {
        Self {
            profile_id: &value.profile_id,
        }
    }
}

#[derive(Debug)]
pub struct UpdateParams<'a> {
    pub id: &'a Uuid,
    pub profile_id: &'a Uuid,
    pub custom_title: Option<&'a str>,
}
