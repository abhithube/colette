use chrono::{DateTime, Utc};
use colette_core::bookmarks::BookmarkFindManyParams;

#[derive(Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: &'a str,
    pub limit: i64,
    pub published_at: Option<&'a DateTime<Utc>>,
    pub should_filter: bool,
    pub collection_id: Option<&'a str>,
}

impl<'a> From<&'a BookmarkFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a BookmarkFindManyParams) -> Self {
        Self {
            profile_id: &value.profile_id,
            limit: value.limit,
            published_at: value.published_at.as_ref(),
            should_filter: value.should_filter,
            collection_id: value.collection_id.as_deref(),
        }
    }
}
