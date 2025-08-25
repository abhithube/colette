use chrono::{DateTime, Utc};
use colette_common::RepositoryError;
use colette_ingestion::{
    Feed, FeedFindOutdatedParams, FeedFindParams, FeedId, FeedRepository, FeedStatus,
    FeedUpsertParams,
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

#[async_trait::async_trait]
impl FeedRepository for PostgresFeedRepository {
    async fn find(&self, params: FeedFindParams) -> Result<Vec<Feed>, RepositoryError> {
        let feeds = sqlx::query_file_as!(
            FeedRow,
            "queries/feeds/find.sql",
            params.id.map(|e| e.as_inner()),
            params.cursor.map(DbUrl) as Option<DbUrl>,
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(feeds)
    }

    async fn find_by_source_url(&self, source_url: Url) -> Result<Option<Feed>, RepositoryError> {
        let feed = sqlx::query_file_as!(
            FeedRow,
            "queries/feeds/find_by_source_url.sql",
            DbUrl(source_url) as DbUrl
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

    async fn upsert(&self, params: FeedUpsertParams) -> Result<FeedId, RepositoryError> {
        let mut fe_links = Vec::<DbUrl>::new();
        let mut fe_titles = Vec::<String>::new();
        let mut fe_published_ats = Vec::<DateTime<Utc>>::new();
        let mut fe_descriptions = Vec::<Option<String>>::new();
        let mut fe_authors = Vec::<Option<String>>::new();
        let mut fe_thumbnail_urls = Vec::<Option<DbUrl>>::new();

        for item in params.feed_entry_items {
            fe_links.push(DbUrl(item.link));
            fe_titles.push(item.title);
            fe_published_ats.push(item.published_at);
            fe_descriptions.push(item.description);
            fe_authors.push(item.author);
            fe_thumbnail_urls.push(item.thumbnail_url.map(Into::into));
        }

        let id = sqlx::query_file_scalar!(
            "queries/feeds/upsert.sql",
            DbUrl(params.source_url) as DbUrl,
            DbUrl(params.link) as DbUrl,
            params.title,
            params.description,
            params.refresh_interval_min as i32,
            params.is_custom,
            &fe_links as &[DbUrl],
            &fe_titles,
            &fe_published_ats,
            &fe_descriptions as &[Option<String>],
            &fe_authors as &[Option<String>],
            &fe_thumbnail_urls as &[Option<DbUrl>]
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id.into())
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
    refresh_interval_min: i32,
    status: DbFeedStatus,
    refreshed_at: Option<DateTime<Utc>>,
    is_custom: bool,
}

impl From<FeedRow> for Feed {
    fn from(value: FeedRow) -> Self {
        Self {
            id: value.id.into(),
            source_url: value.source_url.into(),
            link: value.link.0,
            title: value.title,
            description: value.description,
            refreshed_at: value.refreshed_at,
            refresh_interval_min: value.refresh_interval_min as u32,
            status: value.status.into(),
            is_custom: value.is_custom,
        }
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
