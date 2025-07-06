use chrono::Utc;
use colette_core::{
    Feed, Subscription, Tag,
    subscription::{Error, ImportSubscriptionsData, SubscriptionParams, SubscriptionRepository},
    tag::TagParams,
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    feed::{FeedBase, FeedInsert},
    subscription::{SubscriptionBase, SubscriptionDelete, SubscriptionInsert, SubscriptionSelect},
    subscription_tag::{SubscriptionTagBase, SubscriptionTagDelete, SubscriptionTagInsert},
    tag::{TagBase, TagInsert},
};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder as _;
use uuid::Uuid;

use super::{IdRow, PreparedClient as _, SqliteRow};

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
                let (sql, values) = SubscriptionSelect::sqlite(params)
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

                let mut stack: Vec<(Option<Uuid>, colette_opml::Outline)> = data
                    .outlines
                    .into_iter()
                    .map(|outline| (None, outline))
                    .collect();

                while let Some((parent_id, outline)) = stack.pop() {
                    if !outline.outline.is_empty() {
                        let tag_id = {
                            let (sql, values) = TagParams {
                                title: Some(outline.text.clone()),
                                user_id: Some(data.user_id),
                                ..Default::default()
                            }
                            .into_select()
                            .build_rusqlite(SqliteQueryBuilder);
                            let tag = tx.query_opt_prepared::<Tag>(&sql, &values)?;

                            match tag {
                                Some(tag) => tag.id,
                                _ => {
                                    let (sql, values) = TagInsert {
                                        tags: [TagBase {
                                            id: Uuid::new_v4(),
                                            title: &outline.text,
                                            created_at: Utc::now(),
                                            updated_at: Utc::now(),
                                        }],
                                        user_id: data.user_id,
                                        upsert: true,
                                    }
                                    .into_insert()
                                    .build_rusqlite(SqliteQueryBuilder);
                                    let row = tx.query_one_prepared::<IdRow>(&sql, &values)?;

                                    row.id
                                }
                            }
                        };

                        for child in outline.outline {
                            stack.push((Some(tag_id), child));
                        }
                    } else if let (Some(link), Some(xml_url)) = (outline.html_url, outline.xml_url)
                    {
                        let title = outline.title.unwrap_or(outline.text);

                        let feed = FeedInsert {
                            feeds: [FeedBase {
                                id: Uuid::new_v4(),
                                source_url: &xml_url,
                                link: &link,
                                title: &title,
                                description: None,
                                refreshed_at: None,
                                is_custom: false,
                            }],
                            upsert: false,
                        };

                        let (sql, values) = feed.into_insert().build_rusqlite(SqliteQueryBuilder);
                        let row = tx.query_one_prepared::<IdRow>(&sql, &values)?;

                        let subscription_id = {
                            let (sql, values) = SubscriptionInsert {
                                subscriptions: [SubscriptionBase {
                                    id: Uuid::new_v4(),
                                    title: &title,
                                    description: None,
                                    feed_id: row.id,
                                    created_at: Utc::now(),
                                    updated_at: Utc::now(),
                                }],
                                user_id: data.user_id,
                                upsert: true,
                            }
                            .into_insert()
                            .build_rusqlite(SqliteQueryBuilder);
                            let row = tx.query_one_prepared::<IdRow>(&sql, &values)?;

                            row.id
                        };

                        if let Some(tag_id) = parent_id {
                            let subscription_tag = SubscriptionTagInsert {
                                subscription_tags: [SubscriptionTagBase {
                                    subscription_id,
                                    user_id: data.user_id,
                                    tag_ids: vec![tag_id],
                                }],
                            };

                            let (sql, values) = subscription_tag
                                .into_insert()
                                .build_rusqlite(SqliteQueryBuilder);

                            tx.execute_prepared(&sql, &values)?;
                        }
                    }
                }

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
