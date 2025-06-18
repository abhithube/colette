use chrono::Utc;
use colette_core::{
    Feed, Subscription, Tag,
    subscription::{Error, ImportSubscriptionsData, SubscriptionParams, SubscriptionRepository},
    tag::TagParams,
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    feed::FeedInsert,
    subscription::{SubscriptionDelete, SubscriptionInsert},
    subscription_tag::{SubscriptionTagDelete, SubscriptionTagInsert},
    tag::TagInsert,
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder as _;
use tokio_postgres::{error::SqlState, types::Json};
use uuid::Uuid;

use super::{IdRow, PgRow, PreparedClient as _};

#[derive(Debug, Clone)]
pub struct PostgresSubscriptionRepository {
    pool: Pool,
}

impl PostgresSubscriptionRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionRepository for PostgresSubscriptionRepository {
    async fn query(&self, params: SubscriptionParams) -> Result<Vec<Subscription>, Error> {
        let client = self.pool.get().await?;

        let (sql, values) = params.into_select().build_postgres(PostgresQueryBuilder);
        let subscriptions = client.query_prepared::<Subscription>(&sql, &values).await?;

        Ok(subscriptions)
    }

    async fn save(&self, data: &Subscription) -> Result<(), Error> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;

        {
            let (sql, values) = SubscriptionInsert {
                id: data.id,
                title: &data.title,
                description: data.description.as_deref(),
                feed_id: data.feed_id,
                user_id: data.user_id,
                created_at: data.created_at,
                updated_at: data.updated_at,
                upsert: false,
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);

            tx.execute_prepared(&sql, &values)
                .await
                .map_err(|e| match e.code() {
                    Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.feed_id),
                    _ => Error::PostgresClient(e),
                })?;
        }

        if let Some(ref tags) = data.tags {
            let (sql, values) = SubscriptionTagDelete {
                subscription_id: data.id,
                tag_ids: tags.iter().map(|e| e.id),
            }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

            tx.execute_prepared(&sql, &values).await?;

            if !tags.is_empty() {
                let (sql, values) = SubscriptionTagInsert {
                    subscription_id: data.id,
                    user_id: data.user_id,
                    tag_ids: tags.iter().map(|e| e.id),
                }
                .into_insert()
                .build_postgres(PostgresQueryBuilder);

                tx.execute_prepared(&sql, &values).await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = SubscriptionDelete { id }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

        client.execute_prepared(&sql, &values).await?;

        Ok(())
    }

    async fn import(&self, data: ImportSubscriptionsData) -> Result<(), Error> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;

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
                    .build_postgres(PostgresQueryBuilder);
                    let tag = tx.query_opt_prepared::<Tag>(&sql, &values).await?;

                    match tag {
                        Some(tag) => tag.id,
                        _ => {
                            let (sql, values) = TagInsert {
                                id: Uuid::new_v4(),
                                title: &outline.text,
                                user_id: data.user_id,
                                created_at: Utc::now(),
                                updated_at: Utc::now(),
                                upsert: true,
                            }
                            .into_insert()
                            .build_postgres(PostgresQueryBuilder);
                            let row = tx.query_one_prepared::<IdRow>(&sql, &values).await?;

                            row.id
                        }
                    }
                };

                for child in outline.outline {
                    stack.push((Some(tag_id), child));
                }
            } else if let (Some(link), Some(xml_url)) = (outline.html_url, outline.xml_url) {
                let title = outline.title.unwrap_or(outline.text);

                let feed = FeedInsert {
                    id: Uuid::new_v4(),
                    source_url: &xml_url,
                    link: &link,
                    title: &title,
                    description: None,
                    refreshed_at: None,
                    is_custom: false,
                };

                let (sql, values) = feed.into_insert().build_postgres(PostgresQueryBuilder);
                let feed = tx.query_one_prepared::<Feed>(&sql, &values).await?;

                let subscription_id = {
                    let (sql, values) = SubscriptionInsert {
                        id: Uuid::new_v4(),
                        title: &title,
                        description: None,
                        feed_id: feed.id,
                        user_id: data.user_id,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        upsert: true,
                    }
                    .into_insert()
                    .build_postgres(PostgresQueryBuilder);
                    let row = tx.query_one_prepared::<IdRow>(&sql, &values).await?;

                    row.id
                };

                if let Some(tag_id) = parent_id {
                    let subscription_tag = SubscriptionTagInsert {
                        subscription_id,
                        user_id: data.user_id,
                        tag_ids: vec![tag_id],
                    };

                    let (sql, values) = subscription_tag
                        .into_insert()
                        .build_postgres(PostgresQueryBuilder);

                    tx.execute_prepared(&sql, &values).await?;
                }
            }
        }

        tx.commit().await?;

        Ok(())
    }
}

impl From<PgRow<'_>> for Subscription {
    fn from(PgRow(value): PgRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            title: value.get("title"),
            description: value.get("description"),
            feed_id: value.get("feed_id"),
            user_id: value.get("user_id"),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
            feed: value.try_get::<_, String>("link").ok().map(|link| Feed {
                id: value.get("feed_id"),
                source_url: value.get::<_, String>("source_url").parse().unwrap(),
                link: link.parse().unwrap(),
                title: value.get("feed_title"),
                description: value.get("description"),
                refreshed_at: value.get("refreshed_at"),
                is_custom: value.get("is_custom"),
                entries: None,
            }),
            tags: value.try_get::<_, Json<Vec<Tag>>>("tags").map(|e| e.0).ok(),
            unread_count: value.try_get("unread_count").ok(),
        }
    }
}
