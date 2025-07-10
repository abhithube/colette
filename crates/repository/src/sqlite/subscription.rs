use std::collections::HashMap;

use colette_core::{
    Feed, Subscription, Tag,
    subscription::{Error, ImportSubscriptionsData, SubscriptionParams, SubscriptionRepository},
};
use colette_query::{
    Dialect, IntoDelete, IntoInsert, IntoSelect,
    feed::{FeedBase, FeedInsert, FeedSelect},
    subscription::{SubscriptionBase, SubscriptionDelete, SubscriptionInsert, SubscriptionSelect},
    subscription_tag::{SubscriptionTagBase, SubscriptionTagDelete, SubscriptionTagInsert},
    tag::{TagBase, TagInsert, TagSelect},
};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder as _;
use url::Url;
use uuid::Uuid;

use super::{PreparedClient as _, SqliteRow};

#[derive(Debug, Clone)]
pub struct SqliteSubscriptionRepository {
    pool: Pool,
}

impl SqliteSubscriptionRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionRepository for SqliteSubscriptionRepository {
    async fn query(&self, params: SubscriptionParams) -> Result<Vec<Subscription>, Error> {
        let client = self.pool.get().await?;

        let subscriptions = client
            .interact(move |conn| {
                let (sql, values) = SubscriptionSelect {
                    id: params.id,
                    tags: params.tags,
                    user_id: params.user_id,
                    cursor: params.cursor.as_ref().map(|(x, y)| (x.as_str(), *y)),
                    limit: params.limit,
                    with_feed: params.with_feed,
                    with_unread_count: params.with_unread_count,
                    with_tags: params.with_tags,
                    dialect: Dialect::Sqlite,
                    ..Default::default()
                }
                .into_select()
                .build_rusqlite(SqliteQueryBuilder);
                conn.query_prepared::<Subscription>(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(subscriptions)
    }

    async fn save(&self, data: &Subscription) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let data = data.to_owned();

        client
            .interact(move |conn| {
                let tx = conn.transaction()?;

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
                    .build_rusqlite(SqliteQueryBuilder);

                    tx.execute_prepared(&sql, &values).map_err(|e| {
                        match e.sqlite_error().map(|e| e.extended_code) {
                            Some(rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE) => {
                                Error::Conflict(data.feed_id)
                            }
                            _ => Error::SqliteClient(e),
                        }
                    })?;
                }

                if let Some(ref tags) = data.tags {
                    let (sql, values) = SubscriptionTagDelete {
                        subscription_id: data.id,
                        tag_ids: tags.iter().map(|e| e.id),
                    }
                    .into_delete()
                    .build_rusqlite(SqliteQueryBuilder);

                    tx.execute_prepared(&sql, &values)?;

                    if !tags.is_empty() {
                        let (sql, values) = SubscriptionTagInsert {
                            subscription_tags: [SubscriptionTagBase {
                                subscription_id: data.id,
                                user_id: data.user_id,
                                tag_ids: tags.iter().map(|e| e.id),
                            }],
                        }
                        .into_insert()
                        .build_rusqlite(SqliteQueryBuilder);

                        tx.execute_prepared(&sql, &values)?;
                    }
                }

                tx.commit()?;

                Ok::<_, Error>(())
            })
            .await
            .unwrap()?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        client
            .interact(move |conn| {
                let (sql, values) = SubscriptionDelete {
                    id: Some(id),
                    ..Default::default()
                }
                .into_delete()
                .build_rusqlite(SqliteQueryBuilder);
                conn.execute_prepared(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(())
    }

    async fn import(&self, data: ImportSubscriptionsData) -> Result<(), Error> {
        let client = self.pool.get().await?;

        client
            .interact(move |conn| {
                let tx = conn.transaction()?;

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

                    let (sql, values) = TagInsert {
                        tags,
                        user_id: data.user_id,
                        upsert: true,
                    }
                    .into_insert()
                    .build_rusqlite(SqliteQueryBuilder);
                    tx.execute_prepared(&sql, &values)?;

                    let (sql, values) = TagSelect {
                        titles: Some(titles),
                        user_id: Some(data.user_id),
                        ..Default::default()
                    }
                    .into_select()
                    .build_rusqlite(SqliteQueryBuilder);
                    let tags = tx.query_prepared::<Tag>(&sql, &values)?;

                    tags.into_iter()
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
                                refreshed_at: feed.refreshed_at,
                                is_custom: feed.is_custom,
                            });

                            source_urls.push(feed.source_url.clone());
                        }
                    }

                    let (sql, values) = FeedInsert {
                        feeds,
                        upsert: false,
                    }
                    .into_insert()
                    .build_rusqlite(SqliteQueryBuilder);
                    tx.execute_prepared(&sql, &values)?;

                    let (sql, values) = FeedSelect {
                        source_urls: Some(source_urls.iter().map(|e| e.as_str()).collect()),
                        ..Default::default()
                    }
                    .into_select()
                    .build_rusqlite(SqliteQueryBuilder);
                    let feeds = tx.query_prepared::<Feed>(&sql, &values)?;

                    feeds
                        .into_iter()
                        .map(|e| (e.source_url.clone(), e.id))
                        .collect::<HashMap<_, _>>()
                };

                let mut subscription_map = {
                    let (sql, values) = SubscriptionSelect {
                        user_id: Some(data.user_id),
                        feeds: Some(feed_map.values().copied().collect()),
                        dialect: Dialect::Sqlite,
                        ..Default::default()
                    }
                    .into_select()
                    .build_rusqlite(SqliteQueryBuilder);
                    let subscriptions = tx.query_prepared::<Subscription>(&sql, &values)?;

                    subscriptions
                        .into_iter()
                        .flat_map(|s| s.feed.map(|f| (f.source_url, s.id)))
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
                            && let Some(subscription_id) =
                                subscription_map.get(&feed.source_url).copied()
                        {
                            let tag_ids = tag
                                .iter()
                                .flat_map(|e| tag_map.get(&e.title).copied())
                                .collect::<Vec<_>>();

                            subscription_tags.push(SubscriptionTagBase {
                                subscription_id,
                                user_id: data.user_id,
                                tag_ids,
                            });
                        }
                    }

                    let (sql, values) = SubscriptionInsert {
                        subscriptions,
                        user_id: data.user_id,
                        upsert: false,
                    }
                    .into_insert()
                    .build_rusqlite(SqliteQueryBuilder);
                    tx.execute_prepared(&sql, &values)?;

                    let (sql, values) = SubscriptionTagInsert { subscription_tags }
                        .into_insert()
                        .build_rusqlite(SqliteQueryBuilder);
                    tx.execute_prepared(&sql, &values)?;
                };

                tx.commit()?;

                Ok::<_, Error>(())
            })
            .await
            .unwrap()?;

        Ok(())
    }
}

impl From<SqliteRow<'_>> for Subscription {
    fn from(SqliteRow(value): SqliteRow<'_>) -> Self {
        Self {
            id: value.get_unwrap("id"),
            title: value.get_unwrap("title"),
            description: value.get_unwrap("description"),
            feed_id: value.get_unwrap("feed_id"),
            user_id: value.get_unwrap("user_id"),
            created_at: value.get_unwrap("created_at"),
            updated_at: value.get_unwrap("updated_at"),
            feed: value.get::<_, String>("link").ok().map(|link| Feed {
                id: value.get_unwrap("feed_id"),
                source_url: value.get_unwrap::<_, String>("source_url").parse().unwrap(),
                link: link.parse().unwrap(),
                title: value.get_unwrap("feed_title"),
                description: value.get_unwrap("description"),
                refreshed_at: value.get_unwrap("refreshed_at"),
                is_custom: value.get_unwrap("is_custom"),
                entries: None,
            }),
            tags: value
                .get::<_, String>("tags")
                .map(|e| serde_json::from_str(&e).unwrap())
                .ok(),
            unread_count: value.get("unread_count").ok(),
        }
    }
}
