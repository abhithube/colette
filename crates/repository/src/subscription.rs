use chrono::{DateTime, Utc};
use colette_core::{
    Feed, RepositoryError, Subscription, Tag,
    feed::DEFAULT_INTERVAL,
    subscription::{
        ImportSubscriptionsParams, SubscriptionById, SubscriptionFindParams,
        SubscriptionInsertParams, SubscriptionLinkTagParams, SubscriptionRepository,
        SubscriptionUpdateParams,
    },
};
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::{DbUrl, feed::DbFeedStatus};

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
    async fn find(
        &self,
        params: SubscriptionFindParams,
    ) -> Result<Vec<Subscription>, RepositoryError> {
        let (cursor_title, cursor_id) = if let Some((title, id)) = params.cursor {
            (Some(title), Some(id))
        } else {
            (None, None)
        };

        let subscriptions = sqlx::query_file_as!(
            SubscriptionRow,
            "queries/subscriptions/find.sql",
            params.id,
            params.user_id,
            params.tags.as_deref(),
            cursor_title,
            cursor_id,
            params.limit.map(|e| e as i64),
            params.with_unread_count,
            params.with_tags
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(subscriptions)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<SubscriptionById>, RepositoryError> {
        let subscription = sqlx::query_file_as!(
            SubscriptionByIdRow,
            "queries/subscriptions/find_by_id.sql",
            id
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(subscription)
    }

    async fn insert(&self, params: SubscriptionInsertParams) -> Result<Uuid, RepositoryError> {
        let id = sqlx::query_file_scalar!(
            "queries/subscriptions/insert.sql",
            params.title,
            params.description,
            params.feed_id,
            params.user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => RepositoryError::Duplicate,
            _ => RepositoryError::Unknown(e),
        })?;

        Ok(id)
    }

    async fn update(&self, params: SubscriptionUpdateParams) -> Result<(), RepositoryError> {
        let (has_description, description) = if let Some(description) = params.description {
            (true, description)
        } else {
            (false, None)
        };

        sqlx::query_file!(
            "queries/subscriptions/update.sql",
            params.id,
            params.title,
            has_description,
            description,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query_file!("queries/subscriptions/delete_by_id.sql", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn link_tags(&self, params: SubscriptionLinkTagParams) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/subscription_tags/update.sql",
            params.subscription_id,
            &params.tag_ids
        )
        .execute(&self.pool)
        .await?;

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
            params.user_id,
            &feed_source_urls as &[DbUrl],
            &feed_links as &[DbUrl],
            &feed_titles,
            DEFAULT_INTERVAL as i32,
            &tag_titles,
            &st_feed_source_urls as &[DbUrl],
            &st_tag_titles,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

struct SubscriptionRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    feed_id: Uuid,
    source_url: DbUrl,
    link: DbUrl,
    feed_title: String,
    feed_description: Option<String>,
    refresh_interval_min: i32,
    refreshed_at: Option<DateTime<Utc>>,
    status: DbFeedStatus,
    is_custom: bool,
    unread_count: Option<i64>,
    tags: Option<Json<Vec<Tag>>>,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SubscriptionRow> for Subscription {
    fn from(value: SubscriptionRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            description: value.description,
            feed_id: value.feed_id,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            feed: Feed {
                id: value.feed_id,
                source_url: value.source_url.into(),
                link: value.link.into(),
                title: value.feed_title,
                description: value.feed_description,
                refresh_interval_min: value.refresh_interval_min as u32,
                refreshed_at: value.refreshed_at,
                status: value.status.into(),
                is_custom: value.is_custom,
            },
            tags: value.tags.map(|e| e.0),
            unread_count: value.unread_count,
        }
    }
}

struct SubscriptionByIdRow {
    id: Uuid,
    user_id: Uuid,
}

impl From<SubscriptionByIdRow> for SubscriptionById {
    fn from(value: SubscriptionByIdRow) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
        }
    }
}
