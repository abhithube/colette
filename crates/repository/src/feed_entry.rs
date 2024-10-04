use colette_core::{
    common::{Findable, IdParams, Updatable},
    feed_entry::{
        Cursor, Error, FeedEntryFindManyFilters, FeedEntryRepository, FeedEntryUpdateData,
    },
    FeedEntry,
};
use sea_orm::{
    prelude::Uuid, ActiveModelTrait, DatabaseConnection, IntoActiveModel, TransactionError,
    TransactionTrait,
};

use crate::query;

pub struct FeedEntrySqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl FeedEntrySqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for FeedEntrySqlRepository {
    type Params = IdParams;
    type Output = Result<FeedEntry, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(&self.db, params).await
    }
}

#[async_trait::async_trait]
impl Updatable for FeedEntrySqlRepository {
    type Params = IdParams;
    type Data = FeedEntryUpdateData;
    type Output = Result<FeedEntry, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    let Some(model) =
                        query::profile_feed_entry::select_by_id(txn, params.id, params.profile_id)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?
                    else {
                        return Err(Error::NotFound(params.id));
                    };
                    let mut active_model = model.into_active_model();

                    if let Some(has_read) = data.has_read {
                        active_model.has_read.set_if_not_equals(has_read);
                    }

                    if active_model.is_changed() {
                        active_model
                            .update(txn)
                            .await
                            .map_err(|e| Error::Unknown(e.into()))?;
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })?;

        find_by_id(&self.db, params).await
    }
}

#[async_trait::async_trait]
impl FeedEntryRepository for FeedEntrySqlRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<FeedEntryFindManyFilters>,
    ) -> Result<Vec<FeedEntry>, Error> {
        find(&self.db, None, profile_id, limit, cursor, filters).await
    }
}

async fn find(
    db: &DatabaseConnection,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<FeedEntryFindManyFilters>,
) -> Result<Vec<FeedEntry>, Error> {
    let mut feed_id: Option<Uuid> = None;
    let mut smart_feed_id: Option<Uuid> = None;
    let mut has_read: Option<bool> = None;
    let mut tags: Option<Vec<String>> = None;

    if let Some(filters) = filters {
        feed_id = filters.feed_id;
        smart_feed_id = filters.smart_feed_id;
        has_read = filters.has_read;
        tags = filters.tags;
    }

    colette_postgres::profile_feed_entry::select(
        db.get_postgres_connection_pool(),
        id,
        profile_id,
        feed_id,
        has_read,
        tags.as_deref(),
        smart_feed_id,
        cursor,
        limit,
    )
    .await
    .map_err(|e| Error::Unknown(e.into()))
}

async fn find_by_id(db: &DatabaseConnection, params: IdParams) -> Result<FeedEntry, Error> {
    let mut feed_entries = find(db, Some(params.id), params.profile_id, None, None, None).await?;
    if feed_entries.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(feed_entries.swap_remove(0))
}
