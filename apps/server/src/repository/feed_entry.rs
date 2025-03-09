use colette_core::{
    FeedEntry,
    feed_entry::{Error, FeedEntryFindParams, FeedEntryRepository},
};
use colette_model::FeedEntryRow;
use colette_query::IntoSelect;
use sea_orm::{ConnectionTrait, DatabaseConnection, FromQueryResult};

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
