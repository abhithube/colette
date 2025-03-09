use colette_core::{
    FeedEntry,
    feed_entry::{Error, FeedEntryFindParams, FeedEntryRepository},
};
use colette_query::IntoSelect;
use sea_orm::{ConnectionTrait, DatabaseConnection, FromQueryResult};

use super::common::parse_timestamp;

#[derive(Debug, Clone)]
pub struct SqliteFeedEntryRepository {
    db: DatabaseConnection,
}

impl SqliteFeedEntryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl FeedEntryRepository for SqliteFeedEntryRepository {
    async fn find_feed_entries(
        &self,
        params: FeedEntryFindParams,
    ) -> Result<Vec<FeedEntry>, Error> {
        let feed_entries = FeedEntryRow::find_by_statement(
            self.db.get_database_backend().build(&params.into_select()),
        )
        .all(&self.db)
        .await
        .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(feed_entries)
    }
}

#[derive(sea_orm::FromQueryResult)]
pub(crate) struct FeedEntryRow {
    pub(crate) id: String,
    pub(crate) link: String,
    pub(crate) title: String,
    pub(crate) published_at: i32,
    pub(crate) description: Option<String>,
    pub(crate) author: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) feed_id: String,
}

impl From<FeedEntryRow> for FeedEntry {
    fn from(value: FeedEntryRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            link: value.link.parse().unwrap(),
            title: value.title,
            published_at: parse_timestamp(value.published_at).unwrap(),
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url.and_then(|e| e.parse().ok()),
            feed_id: value.feed_id.parse().unwrap(),
        }
    }
}
