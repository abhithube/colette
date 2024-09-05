use colette_core::{
    common::{Findable, IdParams, Updatable},
    feed_entry::{
        Cursor, Error, FeedEntryFindManyFilters, FeedEntryRepository, FeedEntryUpdateData,
    },
    FeedEntry,
};
use colette_entity::PfeWithFe;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, IntoActiveModel, TransactionError,
    TransactionTrait,
};
use uuid::Uuid;

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
            .transaction::<_, FeedEntry, Error>(|txn| {
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

                    find_by_id(txn, params).await
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Transaction(e) => e,
                _ => Error::Unknown(e.into()),
            })
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

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<FeedEntryFindManyFilters>,
) -> Result<Vec<FeedEntry>, Error> {
    let models =
        query::profile_feed_entry::select_with_entry(db, id, profile_id, limit, cursor, filters)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

    let feed_entries = models
        .into_iter()
        .filter_map(|(pfe, fe_opt)| fe_opt.map(|fe| FeedEntry::from(PfeWithFe { pfe, fe })))
        .collect::<Vec<_>>();

    Ok(feed_entries)
}

async fn find_by_id<Db: ConnectionTrait>(db: &Db, params: IdParams) -> Result<FeedEntry, Error> {
    let feed_entries = find(db, Some(params.id), params.profile_id, None, None, None).await?;

    feed_entries
        .first()
        .cloned()
        .ok_or(Error::NotFound(params.id))
}
