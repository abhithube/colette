use uuid::Uuid;

use super::{Cursor, Error, FeedEntry};
use crate::common::{Findable, IdParams, NonEmptyString, Updatable};

pub trait FeedEntryRepository:
    Findable<Params = FeedEntryFindParams, Output = Result<Vec<FeedEntry>, Error>>
    + Updatable<Params = IdParams, Data = FeedEntryUpdateData, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
}

#[derive(Clone, Debug, Default)]
pub struct FeedEntryFindParams {
    pub id: Option<Uuid>,
    pub feed_id: Option<Uuid>,
    pub smart_feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
    pub tags: Option<Vec<NonEmptyString>>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Clone, Debug, Default)]
pub struct FeedEntryUpdateData {
    pub has_read: Option<bool>,
}
