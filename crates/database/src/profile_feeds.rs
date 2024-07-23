use colette_core::feeds::FeedsFindManyParams;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: &'a Uuid,
}

impl<'a> From<&'a FeedsFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a FeedsFindManyParams) -> Self {
        Self {
            profile_id: &value.profile_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpdateParams<'a> {
    pub id: &'a Uuid,
    pub profile_id: &'a Uuid,
    pub custom_title: Option<&'a str>,
}
