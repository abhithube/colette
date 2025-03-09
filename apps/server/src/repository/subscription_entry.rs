use colette_core::{
    SubscriptionEntry,
    common::Transaction,
    subscription_entry::{
        Error, SubscriptionEntryById, SubscriptionEntryFindByIdParams, SubscriptionEntryFindParams,
        SubscriptionEntryRepository,
    },
};
use colette_model::SubscriptionEntryRow;
use colette_query::IntoSelect;
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, FromQueryResult};

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
