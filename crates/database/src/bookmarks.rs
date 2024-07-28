use chrono::{DateTime, Utc};
use colette_core::bookmarks::BookmarksFindManyParams;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: Uuid,
    pub limit: i64,
    pub published_at: Option<&'a DateTime<Utc>>,
    pub should_filter: bool,
    pub collection_id: Option<Uuid>,
}

impl<'a> From<&'a BookmarksFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a BookmarksFindManyParams) -> Self {
        Self {
            profile_id: value.profile_id,
            limit: value.limit,
            published_at: value.published_at.as_ref(),
            should_filter: value.should_filter,
            collection_id: value.collection_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpdateParams<'a> {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub custom_title: Option<&'a str>,
    pub custom_thumbnail_url: Option<&'a str>,
    pub custom_published_at: Option<&'a DateTime<Utc>>,
    pub custom_author: Option<&'a str>,
}