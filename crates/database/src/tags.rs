use colette_core::bookmarks::BookmarksFindManyParams;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: &'a Uuid,
}

impl<'a> From<&'a BookmarksFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a BookmarksFindManyParams) -> Self {
        Self {
            profile_id: &value.profile_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpdateParams<'a> {
    pub id: &'a Uuid,
    pub profile_id: &'a Uuid,
    pub title: Option<&'a str>,
}
