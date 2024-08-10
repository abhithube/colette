use chrono::{DateTime, Utc};
use colette_core::{
    common::FindOneParams,
    entries::{EntriesFindManyParams, EntriesRepository, EntriesUpdateData, Error},
};
use colette_entities::profile_feed_entry;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, TransactionError,
    TransactionTrait,
};
use uuid::Uuid;

use crate::PostgresRepository;

#[async_trait::async_trait]
impl EntriesRepository for PostgresRepository {
    async fn find_many_entries(
        &self,
        params: EntriesFindManyParams,
    ) -> Result<Vec<colette_core::Entry>, Error> {
        sqlx::query_file_as!(
            Entry,
            "queries/entries/find_many.sql",
            params.profile_id,
            params.limit,
            params.published_at,
            params.feed_id,
            params.has_read,
            params.tags.as_deref()
        )
        .fetch_all(self.db.get_postgres_connection_pool())
        .await
        .map(|e| e.into_iter().map(colette_core::Entry::from).collect())
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn find_one_entry(&self, params: FindOneParams) -> Result<colette_core::Entry, Error> {
        sqlx::query_file_as!(
            Entry,
            "queries/entries/find_one.sql",
            params.id,
            params.profile_id,
        )
        .fetch_one(self.db.get_postgres_connection_pool())
        .await
        .map(colette_core::Entry::from)
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }

    async fn update_entry(
        &self,
        params: FindOneParams,
        data: EntriesUpdateData,
    ) -> Result<colette_core::Entry, Error> {
        self.db
            .transaction::<_, (), Error>(|txn| {
                Box::pin(async move {
                    let Some(model) = profile_feed_entry::Entity::find_by_id(params.id)
                        .filter(profile_feed_entry::Column::ProfileId.eq(params.profile_id))
                        .one(txn)
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

        self.find_one_entry(params).await
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub published_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
    pub has_read: bool,
    pub feed_id: Uuid,
}

impl From<Entry> for colette_core::Entry {
    fn from(value: Entry) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            published_at: value.published_at,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url,
            has_read: value.has_read,
            feed_id: value.feed_id,
        }
    }
}
