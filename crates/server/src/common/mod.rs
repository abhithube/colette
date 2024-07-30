use std::sync::Arc;

use axum::extract::FromRef;
use colette_core::{
    auth::AuthService, bookmarks::BookmarksService, common, entries::EntriesService,
    feeds::FeedsService, profiles::ProfilesService, tags::TagsService,
};
pub use error::{BaseError, Error};
pub use session::{Session, SESSION_KEY};
use uuid::Uuid;

use crate::{
    bookmarks::Bookmark,
    entries::Entry,
    feeds::{Feed, FeedDetected},
    profiles::Profile,
    tags::Tag,
};

mod error;
mod session;

#[derive(Clone, FromRef)]
pub struct Context {
    pub auth_service: Arc<AuthService>,
    pub bookmark_service: Arc<BookmarksService>,
    pub entries_service: Arc<EntriesService>,
    pub feeds_service: Arc<FeedsService>,
    pub profiles_service: Arc<ProfilesService>,
    pub tags_service: Arc<TagsService>,
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(names("id"))]
pub struct Id(pub Uuid);

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[aliases(BookmarkList = Paginated<Bookmark>, FeedDetectedList = Paginated<FeedDetected>, EntryList = Paginated<Entry>, FeedList = Paginated<Feed>, ProfileList = Paginated<Profile>, TagList = Paginated<Tag>)]
#[serde(rename_all = "camelCase")]
pub struct Paginated<T: serde::Serialize> {
    pub has_more: bool,
    pub data: Vec<T>,
}

impl<T, U> From<common::Paginated<U>> for Paginated<T>
where
    T: From<U> + serde::Serialize,
{
    fn from(value: common::Paginated<U>) -> Self {
        Self {
            has_more: value.has_more,
            data: value.data.into_iter().map(T::from).collect(),
        }
    }
}
