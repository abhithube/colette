use colette_core::collections::{CollectionFindManyParams, CollectionUpdateData};
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
pub struct UpdateData<'a> {
    pub title: Option<&'a str>,
}

impl<'a> From<&'a CollectionUpdateData> for UpdateData<'a> {
    fn from(value: &'a CollectionUpdateData) -> Self {
        Self {
            title: value.title.as_deref(),
        }
    }
}
