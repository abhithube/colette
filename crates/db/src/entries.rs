use chrono::{DateTime, Utc};
use colette_core::{
    common::FindOneParams,
    entries::{EntriesFindManyParams, EntriesRepository, EntriesUpdateData, Error},
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
        .fetch_all(&self.pool)
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
        .fetch_one(&self.pool)
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
        sqlx::query_file_as!(
            Entry,
            "queries/entries/update.sql",
            params.id,
            params.profile_id,
            data.has_read
        )
        .fetch_one(&self.pool)
        .await
        .map(colette_core::Entry::from)
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
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
