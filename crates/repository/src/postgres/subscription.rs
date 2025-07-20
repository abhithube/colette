use std::collections::HashMap;

use chrono::{DateTime, Utc};
use colette_core::{
    Feed, Subscription, Tag,
    subscription::{Error, ImportSubscriptionsData, SubscriptionParams, SubscriptionRepository},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    feed::{FeedBase, FeedInsert, FeedSelect},
    subscription::{SubscriptionBase, SubscriptionDelete, SubscriptionInsert, SubscriptionSelect},
    subscription_tag::{SubscriptionTagBase, SubscriptionTagDelete, SubscriptionTagInsert},
    tag::{TagBase, TagInsert, TagSelect},
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder as _;
use sqlx::{PgPool, types::Json};
use url::Url;
use uuid::Uuid;

use crate::postgres::{DbUrl, feed::FeedRow, tag::TagRow};

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
    async fn query(&self, params: SubscriptionParams) -> Result<Vec<Subscription>, Error> {
        let (sql, values) = SubscriptionSelect {
            id: params.id,
            tags: params.tags,
            user_id: params.user_id,
            cursor: params.cursor.as_ref().map(|(x, y)| (x.as_str(), *y)),
            limit: params.limit.map(|e| e as u64),
            with_feed: params.with_feed,
            with_unread_count: params.with_unread_count,
            with_tags: params.with_tags,
            ..Default::default()
        }
        .into_select()
        .build_sqlx(PostgresQueryBuilder);
        println!("{sql}");
        let rows = sqlx::query_as_with::<_, SubscriptionRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await
            .inspect_err(|e| println!("{e}"))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn save(&self, data: &Subscription) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        {
            let (sql, values) = SubscriptionInsert {
                subscriptions: [SubscriptionBase {
                    id: data.id,
                    title: &data.title,
                    description: data.description.as_deref(),
                    feed_id: data.feed_id,
                    created_at: data.created_at,
                    updated_at: data.updated_at,
                }],
                user_id: data.user_id,
                upsert: false,
            }
            .into_insert()
            .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::Database(e) if e.is_unique_violation() => {
                        Error::Conflict(data.feed_id)
                    }
                    _ => Error::Sqlx(e),
                })?;
        }

        if let Some(ref tags) = data.tags {
            let (sql, values) = SubscriptionTagDelete {
                subscription_id: data.id,
                tag_ids: tags.iter().map(|e| e.id),
            }
            .into_delete()
            .build_sqlx(PostgresQueryBuilder);
            sqlx::query_with(&sql, values).execute(&mut *tx).await?;

            if !tags.is_empty() {
                let (sql, values) = SubscriptionTagInsert {
                    subscription_tags: [SubscriptionTagBase {
                        subscription_id: data.id,
                        tag_ids: tags.iter().map(|e| e.id),
                    }],
                    user_id: data.user_id,
                }
                .into_insert()
                .build_sqlx(PostgresQueryBuilder);
                sqlx::query_with(&sql, values).execute(&mut *tx).await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = SubscriptionDelete {
            id: Some(id),
            ..Default::default()
        }
        .into_delete()
        .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }

    async fn import(&self, data: ImportSubscriptionsData) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        let tag_map = {
            let mut titles = Vec::<&str>::new();
            let mut tags = Vec::<TagBase>::new();

            for tag in data.tags.iter() {
                titles.push(&tag.title);

                tags.push(TagBase {
                    id: tag.id,
                    title: &tag.title,
                    created_at: tag.created_at,
                    updated_at: tag.updated_at,
                });
            }

            if !tags.is_empty() {
                let (sql, values) = TagInsert {
                    tags,
                    user_id: data.user_id,
                    upsert: true,
                }
                .into_insert()
                .build_sqlx(PostgresQueryBuilder);
                sqlx::query_with(&sql, values).execute(&mut *tx).await?;
            }

            let (sql, values) = TagSelect {
                titles: Some(titles),
                user_id: Some(data.user_id),
                ..Default::default()
            }
            .into_select()
            .build_sqlx(PostgresQueryBuilder);
            let rows = sqlx::query_as_with::<_, TagRow, _>(&sql, values)
                .fetch_all(&mut *tx)
                .await?;

            rows.into_iter()
                .map(|e| (e.title, e.id))
                .collect::<HashMap<_, _>>()
        };

        let feed_map = {
            let mut source_urls = Vec::<Url>::new();
            let mut feeds = Vec::<FeedBase>::new();

            for subscription in data.subscriptions.iter() {
                if let Some(ref feed) = subscription.feed {
                    feeds.push(FeedBase {
                        id: Uuid::new_v4(),
                        source_url: feed.source_url.as_str(),
                        link: feed.link.as_str(),
                        title: &feed.title,
                        description: feed.description.as_deref(),
                        refresh_interval_min: feed.refresh_interval_min as i32,
                        is_refreshing: feed.is_refreshing,
                        refreshed_at: feed.refreshed_at,
                        is_custom: feed.is_custom,
                    });

                    source_urls.push(feed.source_url.clone());
                }
            }

            if !feeds.is_empty() {
                let (sql, values) = FeedInsert {
                    feeds,
                    upsert: false,
                }
                .into_insert()
                .build_sqlx(PostgresQueryBuilder);
                sqlx::query_with(&sql, values).execute(&mut *tx).await?;
            }

            let (sql, values) = FeedSelect {
                source_urls: Some(source_urls.iter().map(|e| e.as_str()).collect()),
                ..Default::default()
            }
            .into_select()
            .build_sqlx(PostgresQueryBuilder);
            let rows = sqlx::query_as_with::<_, FeedRow, _>(&sql, values)
                .fetch_all(&mut *tx)
                .await?;

            rows.into_iter()
                .map(|e| (e.source_url.0, e.id))
                .collect::<HashMap<_, _>>()
        };

        let mut subscription_map = {
            let (sql, values) = SubscriptionSelect {
                user_id: Some(data.user_id),
                feeds: Some(feed_map.values().copied().collect()),
                ..Default::default()
            }
            .into_select()
            .build_sqlx(PostgresQueryBuilder);
            let rows = sqlx::query_as_with::<_, SubscriptionRow, _>(&sql, values)
                .fetch_all(&mut *tx)
                .await?;

            rows.into_iter()
                .flat_map(|s| s.source_url.map(|u| (u.0, s.id)))
                .collect::<HashMap<_, _>>()
        };

        {
            let mut subscriptions = Vec::<SubscriptionBase>::new();
            let mut subscription_tags = Vec::<SubscriptionTagBase<Vec<Uuid>>>::new();

            for subscription in data.subscriptions.iter() {
                let Some(ref feed) = subscription.feed else {
                    continue;
                };

                if !subscription_map.contains_key(&feed.source_url)
                    && let Some(feed_id) = feed_map.get(&feed.source_url).copied()
                {
                    let id = Uuid::new_v4();

                    subscriptions.push(SubscriptionBase {
                        id,
                        title: &subscription.title,
                        description: subscription.description.as_deref(),
                        feed_id,
                        created_at: subscription.created_at,
                        updated_at: subscription.updated_at,
                    });

                    subscription_map.insert(feed.source_url.clone(), id);
                }

                if let Some(tag) = subscription.tags.as_deref()
                    && let Some(subscription_id) = subscription_map.get(&feed.source_url).copied()
                {
                    let tag_ids = tag
                        .iter()
                        .flat_map(|e| tag_map.get(&e.title).copied())
                        .collect::<Vec<_>>();

                    subscription_tags.push(SubscriptionTagBase {
                        subscription_id,
                        tag_ids,
                    });
                }
            }

            if !subscriptions.is_empty() {
                let (sql, values) = SubscriptionInsert {
                    subscriptions,
                    user_id: data.user_id,
                    upsert: false,
                }
                .into_insert()
                .build_sqlx(PostgresQueryBuilder);
                sqlx::query_with(&sql, values).execute(&mut *tx).await?;
            }

            if !subscription_tags.is_empty() {
                let (sql, values) = SubscriptionTagInsert {
                    subscription_tags,
                    user_id: data.user_id,
                }
                .into_insert()
                .build_sqlx(PostgresQueryBuilder);
                sqlx::query_with(&sql, values).execute(&mut *tx).await?;
            }
        };

        tx.commit().await?;

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SubscriptionRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    feed_id: Uuid,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[sqlx(default)]
    tags: Option<Json<Vec<Tag>>>,
    #[sqlx(default)]
    unread_count: Option<i64>,

    #[sqlx(default)]
    source_url: Option<DbUrl>,
    #[sqlx(default)]
    link: Option<DbUrl>,
    #[sqlx(default)]
    feed_title: Option<String>,
    #[sqlx(default)]
    feed_description: Option<String>,
    #[sqlx(default)]
    refresh_interval_min: Option<i32>,
    #[sqlx(default)]
    is_refreshing: Option<bool>,
    #[sqlx(default)]
    refreshed_at: Option<DateTime<Utc>>,
    #[sqlx(default)]
    is_custom: Option<bool>,
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
            feed: if let Some(source_url) = value.source_url.map(|e| e.0)
                && let Some(link) = value.link.map(|e| e.0)
                && let Some(title) = value.feed_title
                && let Some(refresh_interval_min) = value.refresh_interval_min.map(|e| e as u32)
                && let Some(is_refreshing) = value.is_refreshing
                && let Some(is_custom) = value.is_custom
            {
                Some(Feed {
                    id: value.feed_id,
                    source_url,
                    link,
                    title,
                    description: value.feed_description,
                    refresh_interval_min,
                    is_refreshing,
                    refreshed_at: value.refreshed_at,
                    is_custom,
                    entries: None,
                })
            } else {
                None
            },
            tags: value.tags.map(|e| e.0),
            unread_count: value.unread_count,
        }
    }
}
