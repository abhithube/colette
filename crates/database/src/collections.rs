use colette_core::collections::{CollectionCreateData, CollectionFindManyParams};
use nanoid::nanoid;

#[derive(Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: &'a str,
}

impl<'a> From<&'a CollectionFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a CollectionFindManyParams) -> Self {
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

impl<'a> From<&'a CollectionCreateData> for InsertData<'a> {
    fn from(value: &'a CollectionCreateData) -> Self {
        Self {
            id: nanoid!(),
            title: &value.title,
            is_default: false,
            profile_id: &value.profile_id,
        }
    }
}
