use chrono::{DateTime, Utc};
use colette_core::{
    common::{IdParams, Paginated},
    feed_entry::{Error, FeedEntryFindManyFilters, FeedEntryRepository, FeedEntryUpdateData},
    FeedEntry,
};
use colette_entities::PfeWithFe;
use colette_utils::base_64;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, IntoActiveModel, TransactionError,
    TransactionTrait,
};
use uuid::Uuid;

use crate::queries;

pub struct FeedEntrySqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl FeedEntrySqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl FeedEntryRepository for FeedEntrySqlRepository {
    async fn find_many(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor_raw: Option<String>,
        filters: Option<FeedEntryFindManyFilters>,
    ) -> Result<Paginated<FeedEntry>, Error> {
        find(&self.db, None, profile_id, limit, cursor_raw, filters).await
    }

    async fn find_one(&self, params: IdParams) -> Result<FeedEntry, Error> {
        find_by_id(&self.db, params).await
    }

    async fn update(
        &self,
        params: IdParams,
        data: FeedEntryUpdateData,
    ) -> Result<FeedEntry, Error> {
        self.db
            .transaction::<_, FeedEntry, Error>(|txn| {
                Box::pin(async move {
                    let Some(model) = queries::profile_feed_entry::select_by_id(
                        txn,
                        params.id,
                        params.profile_id,
                    )
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

async fn find<Db: ConnectionTrait>(
    db: &Db,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor_raw: Option<String>,
    filters: Option<FeedEntryFindManyFilters>,
) -> Result<Paginated<FeedEntry>, Error> {
    let models = queries::profile_feed_entry::select_with_entry(
        db,
        id,
        profile_id,
        limit.map(|e| e + 1),
        cursor_raw.and_then(|e| base_64::decode::<Cursor>(&e).ok()),
        filters,
    )
    .await
    .map_err(|e| Error::Unknown(e.into()))?;

    let mut feed_entries = models
        .into_iter()
        .filter_map(|(pfe, fe_opt)| fe_opt.map(|fe| FeedEntry::from(PfeWithFe { pfe, fe })))
        .collect::<Vec<_>>();
    let mut cursor: Option<String> = None;

    if let Some(limit) = limit {
        let limit = limit as usize;
        if feed_entries.len() > limit {
            feed_entries = feed_entries.into_iter().take(limit).collect();

            if let Some(last) = feed_entries.last() {
                let c = Cursor {
                    id: last.id,
                    published_at: last.published_at,
                };
                let encoded = base_64::encode(&c)?;

                cursor = Some(encoded);
            }
        }
    }

    Ok(Paginated::<FeedEntry> {
        cursor,
        data: feed_entries,
    })
}

async fn find_by_id<Db: ConnectionTrait>(db: &Db, params: IdParams) -> Result<FeedEntry, Error> {
    let feed_entries = find(db, Some(params.id), params.profile_id, Some(1), None, None).await?;

    feed_entries
        .data
        .first()
        .cloned()
        .ok_or(Error::NotFound(params.id))
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTime<Utc>>,
}
