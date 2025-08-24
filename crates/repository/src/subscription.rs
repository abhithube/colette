use chrono::{DateTime, Utc};
use colette_authentication::UserId;
use colette_common::RepositoryError;
use colette_crud::{
    ImportSubscriptionsParams, Subscription, SubscriptionId, SubscriptionRepository,
};
use colette_handler::{SubscriptionDto, SubscriptionQueryParams, SubscriptionQueryRepository};
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::{DbUrl, tag::TagRow};

#[derive(Debug, Clone)]
pub struct PostgresSubscriptionRepository {
    pool: PgPool,
}

impl PostgresSubscriptionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionRepository for PostgresSubscriptionRepository {
    async fn find_by_id(
        &self,
        id: SubscriptionId,
        user_id: UserId,
    ) -> Result<Option<Subscription>, RepositoryError> {
        let subscription = sqlx::query_file_as!(
            SubscriptionByIdRow,
            "queries/subscriptions/find_by_id.sql",
            id.as_inner(),
            user_id.as_inner()
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(subscription)
    }

    async fn save(&self, data: &Subscription) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/subscriptions/upsert.sql",
            data.id().as_inner(),
            data.title().as_inner(),
            data.description().map(|e| e.as_inner()),
            data.feed_id().as_inner(),
            &data.tags().iter().map(|e| e.as_inner()).collect::<Vec<_>>(),
            data.user_id().as_inner(),
            data.created_at(),
            data.updated_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => RepositoryError::Duplicate,
            _ => RepositoryError::Unknown(e),
        })?;

        Ok(())
    }

    async fn delete_by_id(
        &self,
        id: SubscriptionId,
        user_id: UserId,
    ) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/subscriptions/delete_by_id.sql",
            id.as_inner(),
            user_id.as_inner()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            _ => RepositoryError::Unknown(e),
        })?;

        Ok(())
    }

    async fn import(&self, params: ImportSubscriptionsParams) -> Result<(), RepositoryError> {
        let mut feed_source_urls = Vec::<DbUrl>::new();
        let mut feed_links = Vec::<DbUrl>::new();
        let mut feed_titles = Vec::<String>::new();

        let mut st_feed_source_urls = Vec::<DbUrl>::new();
        let mut st_tag_titles = Vec::<String>::new();

        for item in params.subscription_items {
            let source_url = DbUrl(item.feed_url);

            for title in item.tag_titles {
                st_feed_source_urls.push(source_url.clone());
                st_tag_titles.push(title);
            }

            feed_source_urls.push(source_url);
            feed_links.push(item.feed_link.into());
            feed_titles.push(item.feed_title);
        }

        let mut tag_titles = Vec::<String>::new();

        for title in params.tag_titles {
            tag_titles.push(title);
        }

        sqlx::query_file!(
            "queries/subscriptions/import.sql",
            params.user_id.as_inner(),
            &feed_source_urls as &[DbUrl],
            &feed_links as &[DbUrl],
            &feed_titles,
            params.feed_refresh_interval as i32,
            &tag_titles,
            &st_feed_source_urls as &[DbUrl],
            &st_tag_titles,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

struct SubscriptionByIdRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    feed_id: Uuid,
    tags: Vec<Uuid>,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SubscriptionByIdRow> for Subscription {
    fn from(value: SubscriptionByIdRow) -> Self {
        Self::from_unchecked(
            value.id,
            value.title,
            value.description,
            value.feed_id,
            value.tags,
            value.user_id,
            value.created_at,
            value.updated_at,
        )
    }
}

#[async_trait::async_trait]
impl SubscriptionQueryRepository for PostgresSubscriptionRepository {
    async fn query(
        &self,
        params: SubscriptionQueryParams,
    ) -> Result<Vec<SubscriptionDto>, RepositoryError> {
        let (cursor_title, cursor_id) = if let Some((title, id)) = params.cursor {
            (Some(title), Some(id))
        } else {
            (None, None)
        };

        let subscriptions = sqlx::query_file_as!(
            SubscriptionRow,
            "queries/subscriptions/find.sql",
            params.user_id,
            params.id,
            params.tags.as_deref(),
            cursor_title,
            cursor_id,
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(subscriptions)
    }
}

struct SubscriptionRow {
    id: Uuid,
    source_url: DbUrl,
    link: DbUrl,
    title: String,
    description: Option<String>,
    feed_id: Uuid,
    tags: Json<Vec<TagRow>>,
    unread_count: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SubscriptionRow> for SubscriptionDto {
    fn from(value: SubscriptionRow) -> Self {
        Self {
            id: value.id,
            source_url: value.source_url.into(),
            link: value.link.into(),
            title: value.title,
            description: value.description,
            feed_id: value.feed_id,
            tags: value.tags.0.into_iter().map(Into::into).collect(),
            unread_count: value.unread_count,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
