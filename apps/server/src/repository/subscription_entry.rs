use colette_core::{
    SubscriptionEntry,
    common::Transaction,
    subscription_entry::{
        Error, SubscriptionEntryById, SubscriptionEntryFindByIdParams, SubscriptionEntryFindParams,
        SubscriptionEntryRepository,
    },
};
use colette_query::IntoSelect;
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, FromQueryResult};

use super::feed_entry::FeedEntryRow;

#[derive(Debug, Clone)]
pub struct SqliteSubscriptionEntryRepository {
    db: DatabaseConnection,
}

impl SqliteSubscriptionEntryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl SubscriptionEntryRepository for SqliteSubscriptionEntryRepository {
    async fn find_subscription_entries(
        &self,
        params: SubscriptionEntryFindParams,
    ) -> Result<Vec<SubscriptionEntry>, Error> {
        let subscription_entries = SubscriptionEntryRow::find_by_statement(
            self.db.get_database_backend().build(&params.into_select()),
        )
        .all(&self.db)
        .await
        .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(subscription_entries)
    }

    async fn find_subscription_entry_by_id(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionEntryFindByIdParams,
    ) -> Result<SubscriptionEntryById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let id = params.feed_entry_id;

        let Some(result) = tx
            .query_one(self.db.get_database_backend().build(&params.into_select()))
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(SubscriptionEntryById {
            feed_entry_id: result
                .try_get_by_index::<String>(0)
                .unwrap()
                .parse()
                .unwrap(),
            user_id: result
                .try_get_by_index::<String>(1)
                .unwrap()
                .parse()
                .unwrap(),
        })
    }
}

#[derive(sea_orm::FromQueryResult)]
struct SubscriptionEntryRow {
    id: String,
    link: String,
    title: String,
    published_at: i32,
    description: Option<String>,
    author: Option<String>,
    thumbnail_url: Option<String>,
    feed_id: String,

    subscription_id: String,
    user_id: String,
    has_read: bool,
}

impl From<SubscriptionEntryRow> for SubscriptionEntry {
    fn from(value: SubscriptionEntryRow) -> Self {
        Self {
            entry: FeedEntryRow {
                id: value.id,
                link: value.link,
                title: value.title,
                published_at: value.published_at,
                description: value.description,
                author: value.author,
                thumbnail_url: value.thumbnail_url,
                feed_id: value.feed_id,
            }
            .into(),
            subscription_id: value.subscription_id.parse().unwrap(),
            user_id: value.user_id.parse().unwrap(),
            has_read: value.has_read,
        }
    }
}
