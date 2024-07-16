use colette_core::collections::CollectionFindManyParams;
use uuid::Uuid;

#[derive(Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: &'a Uuid,
}

impl<'a> From<&'a CollectionFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a CollectionFindManyParams) -> Self {
        Self {
            profile_id: &value.profile_id,
        }
    }
}

#[derive(Debug)]
pub struct UpdateParams<'a> {
    pub id: &'a Uuid,
    pub profile_id: &'a Uuid,
    pub title: Option<&'a str>,
}
