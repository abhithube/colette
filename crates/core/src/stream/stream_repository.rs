use uuid::Uuid;

use super::{Cursor, Error, FeedEntryFilter, Stream};
use crate::{
    FeedEntry,
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    feed_entry,
};

#[async_trait::async_trait]
pub trait StreamRepository:
    Findable<Params = StreamFindParams, Output = Result<Vec<Stream>, Error>>
    + Creatable<Data = StreamCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = StreamUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
    async fn find_entries(&self, params: StreamEntryFindParams) -> Result<Vec<FeedEntry>, Error>;
}

#[derive(Debug, Clone, Default)]
pub struct StreamFindParams {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone)]
pub struct StreamCreateData {
    pub title: String,
    pub filter: FeedEntryFilter,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct StreamUpdateData {
    pub title: Option<String>,
    pub filter: Option<FeedEntryFilter>,
}

#[derive(Debug, Clone)]
pub struct StreamEntryFindParams {
    pub filter: FeedEntryFilter,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<feed_entry::Cursor>,
}
