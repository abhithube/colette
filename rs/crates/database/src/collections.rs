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