use chrono::{DateTime, Utc};
use colette_common::RepositoryError;
use colette_ingestion::{
    Feed, FeedBatch, FeedFindOutdatedParams, FeedId, FeedRepository, FeedStatus,
};
use sqlx::{
    Decode, Encode, PgPool, Postgres, Type,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef},
};
use url::Url;
use uuid::Uuid;

use crate::DbUrl;

#[derive(Debug, Clone)]
pub struct PostgresFeedRepository {
    pool: PgPool,
}

impl PostgresFeedRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl FeedRepository for PostgresFeedRepository {
    async fn find_by_id(&self, id: FeedId) -> Result<Option<Feed>, RepositoryError> {
        let feed = sqlx::query_file_as!(
            FeedRow,
            "queries/feeds/find_by_source_url.sql",
            id.as_inner(),
            Option::<&str>::None
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(feed)
    }

    async fn find_by_source_url(&self, source_url: &Url) -> Result<Option<Feed>, RepositoryError> {
        let feed = sqlx::query_file_as!(
            FeedRow,
            "queries/feeds/find_by_source_url.sql",
            Option::<Uuid>::None,
            DbUrl(source_url.to_owned()) as DbUrl
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(feed)
    }

    async fn find_outdated(
        &self,
        params: FeedFindOutdatedParams,
    ) -> Result<Vec<Feed>, RepositoryError> {
        let feeds = sqlx::query_file_as!(
            FeedRow,
            "queries/feeds/find_outdated.sql",
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(feeds)
    }

    async fn upsert(&self, data: FeedBatch) -> Result<(), RepositoryError> {
        let mut fe_ids = Vec::<Uuid>::new();
        let mut fe_links = Vec::<DbUrl>::new();
        let mut fe_titles = Vec::<String>::new();
        let mut fe_published_ats = Vec::<DateTime<Utc>>::new();
        let mut fe_descriptions = Vec::<Option<String>>::new();
        let mut fe_authors = Vec::<Option<String>>::new();
        let mut fe_thumbnail_urls = Vec::<Option<DbUrl>>::new();
        let mut fe_created_ats = Vec::<DateTime<Utc>>::new();
        let mut fe_updated_ats = Vec::<DateTime<Utc>>::new();

        for item in data.feed_entries {
            fe_ids.push(item.id().as_inner());
            fe_links.push(DbUrl(item.link().to_owned()));
            fe_titles.push(item.title().to_owned());
            fe_published_ats.push(item.published_at());
            fe_descriptions.push(item.description().map(ToOwned::to_owned));
            fe_authors.push(item.author().map(ToOwned::to_owned));
            fe_thumbnail_urls.push(item.thumbnail_url().map(ToOwned::to_owned).map(DbUrl));
            fe_created_ats.push(item.created_at());
            fe_updated_ats.push(item.updated_at());
        }

        sqlx::query_file!(
            "queries/feeds/upsert.sql",
            data.feed.id().as_inner(),
            DbUrl(data.feed.source_url().to_owned()) as DbUrl,
            DbUrl(data.feed.link().to_owned()) as DbUrl,
            data.feed.title(),
            data.feed.description(),
            data.feed.is_custom(),
            DbFeedStatus(data.feed.status().to_owned()) as DbFeedStatus,
            data.feed.last_refreshed_at(),
            data.feed.created_at(),
            data.feed.updated_at(),
            &fe_ids,
            &fe_links as &[DbUrl],
            &fe_titles,
            &fe_published_ats,
            &fe_descriptions as &[Option<String>],
            &fe_authors as &[Option<String>],
            &fe_thumbnail_urls as &[Option<DbUrl>],
            &fe_created_ats,
            &fe_updated_ats,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_as_failed(&self, source_url: Url) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/feeds/mark_as_failed.sql",
            DbUrl(source_url) as DbUrl
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

pub(crate) struct FeedRow {
    id: Uuid,
    source_url: DbUrl,
    link: DbUrl,
    title: String,
    description: Option<String>,
    is_custom: bool,
    status: DbFeedStatus,
    refresh_interval_min: i32,
    last_refreshed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<FeedRow> for Feed {
    fn from(value: FeedRow) -> Self {
        Self::from_unchecked(
            value.id,
            value.source_url.0,
            value.link.0,
            value.title,
            value.description,
            value.is_custom,
            value.status.0,
            value.refresh_interval_min as u32,
            value.last_refreshed_at,
            value.created_at,
            value.updated_at,
        )
    }
}

pub(crate) struct DbFeedStatus(FeedStatus);

impl From<DbFeedStatus> for FeedStatus {
    fn from(value: DbFeedStatus) -> Self {
        value.0
    }
}

impl From<FeedStatus> for DbFeedStatus {
    fn from(value: FeedStatus) -> Self {
        Self(value)
    }
}

impl Type<Postgres> for DbFeedStatus {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("text")
    }
}

impl Encode<'_, Postgres> for DbFeedStatus {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.extend_from_slice(self.0.to_string().as_bytes());

        Ok(IsNull::No)
    }
}

impl Decode<'_, Postgres> for DbFeedStatus {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => str::from_utf8(value.as_bytes()?)?.parse(),
            PgValueFormat::Text => value.as_str()?.parse(),
        }
        .map(DbFeedStatus)
        .map_err(Into::into)
    }
}
