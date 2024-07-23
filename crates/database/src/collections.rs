use colette_core::collections::CollectionsFindManyParams;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: &'a Uuid,
}

impl<'a> From<&'a CollectionsFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a CollectionsFindManyParams) -> Self {
        Self {
            profile_id: &value.profile_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectDefaultParams<'a> {
    pub profile_id: &'a Uuid,
}

#[derive(Clone, Debug)]
pub struct UpdateParams<'a> {
    pub id: &'a Uuid,
    pub profile_id: &'a Uuid,
    pub title: Option<&'a str>,
}
