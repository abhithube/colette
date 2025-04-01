use std::collections::HashMap;

use chrono::{DateTime, Utc};
use colette_core::{
    Feed, Subscription, Tag,
    subscription::{Error, ImportSubscriptionsData, SubscriptionParams, SubscriptionRepository},
    tag::TagParams,
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    feed::FeedInsert,
    feed_entry::UnreadCountSelectMany,
    subscription::{SubscriptionDelete, SubscriptionInsert},
    subscription_tag::{SubscriptionTagDelete, SubscriptionTagInsert, SubscriptionTagSelect},
    tag::TagInsert,
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::{Row, error::SqlState};
use uuid::Uuid;

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

        let with_tags = params.with_tags;
        let with_unread_count = params.with_unread_count;

        let (sql, values) = params.into_select().build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;
        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let mut subscriptions = rows
            .iter()
            .map(|e| Subscription::from(SubscriptionRow(e)))
            .collect::<Vec<_>>();

        let subscription_ids = subscriptions.iter().map(|e| e.id);

        let mut tag_row_map = HashMap::<Uuid, Vec<SubscriptionTagRow>>::new();
        if with_tags {
            let (sql, values) = SubscriptionTagSelect {
                subscription_ids: subscription_ids.clone(),
            }
            .into_select()
            .build_postgres(PostgresQueryBuilder);

            let stmt = client.prepare_cached(&sql).await?;
            let rows = client.query(&stmt, &values.as_params()).await?;

            let tag_rows = rows
                .iter()
                .map(SubscriptionTagRow::from)
                .collect::<Vec<_>>();
            for row in tag_rows {
                tag_row_map
                    .entry(row.subscription_id)
                    .or_default()
                    .push(row);
            }
        }

        let mut unread_count_map = HashMap::<Uuid, i64>::new();
        if with_unread_count {
            let (sql, values) = UnreadCountSelectMany { subscription_ids }
                .into_select()
                .build_postgres(PostgresQueryBuilder);

            let stmt = client.prepare_cached(&sql).await?;
            let rows = client.query(&stmt, &values.as_params()).await?;

            for row in rows {
                unread_count_map
                    .entry(row.get("id"))
                    .insert_entry(row.get("unread_count"));
            }
        }

        for subscription in subscriptions.iter_mut() {
            if with_tags {
                let tags = tag_row_map
                    .remove(&subscription.id)
                    .unwrap_or_default()
                    .into_iter()
                    .map(Into::into)
                    .collect();
                subscription.tags = Some(tags);
            }

            if with_unread_count {
                let unread_count = unread_count_map
                    .remove(&subscription.id)
                    .unwrap_or_default();

                subscription.unread_count = Some(unread_count);
            }
        }

        Ok(subscriptions)
    }

    async fn save(&self, data: &Subscription) -> Result<(), Error> {
        let mut client = self.pool.get().await?;

        let tx = client.transaction().await?;

        {
            let (sql, values) = SubscriptionInsert {
                id: data.id,
                title: &data.title,
                feed_id: data.feed_id,
                user_id: &data.user_id,
                created_at: data.created_at,
                updated_at: data.updated_at,
                upsert: false,
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);

            let stmt = tx.prepare_cached(&sql).await?;
            tx.execute(&stmt, &values.as_params())
                .await
                .map_err(|e| match e.code() {
                    Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.feed_id),
                    _ => Error::Database(e),
                })?;
        }

        if let Some(ref tags) = data.tags {
            let (sql, values) = SubscriptionTagDelete {
                subscription_id: data.id,
                tag_ids: tags.iter().map(|e| e.id),
            }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

            let stmt = tx.prepare_cached(&sql).await?;
            tx.execute(&stmt, &values.as_params()).await?;

            let (sql, values) = SubscriptionTagInsert {
                subscription_id: data.id,
                user_id: &data.user_id,
                tag_ids: tags.iter().map(|e| e.id),
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);

            let stmt = tx.prepare_cached(&sql).await?;
            tx.execute(&stmt, &values.as_params()).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = SubscriptionDelete { id }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        client.execute(&stmt, &values.as_params()).await?;

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
                        user_id: Some(data.user_id.clone()),
                        ..Default::default()
                    }
                    .into_select()
                    .build_postgres(PostgresQueryBuilder);

                    let stmt = tx.prepare_cached(&sql).await?;
                    let row = tx.query_opt(&stmt, &values.as_params()).await?;

                    match row {
                        Some(row) => row.get("id"),
                        _ => {
                            let (sql, values) = TagInsert {
                                id: Uuid::new_v4(),
                                title: &outline.text,
                                user_id: &data.user_id,
                                created_at: Utc::now(),
                                updated_at: Utc::now(),
                                upsert: true,
                            }
                            .into_insert()
                            .build_postgres(PostgresQueryBuilder);

                            let stmt = tx.prepare_cached(&sql).await?;
                            let row = tx.query_one(&stmt, &values.as_params()).await?;

                            row.get("id")
                        }
                    }
                };

                for child in outline.outline {
                    stack.push((Some(tag_id), child));
                }
            } else if let Some(link) = outline.html_url {
                let title = outline.title.unwrap_or(outline.text);

                let feed = FeedInsert {
                    id: Uuid::new_v4(),
                    link: &link,
                    xml_url: outline.xml_url.as_deref(),
                    title: &title,
                    description: None,
                    refreshed_at: None,
                };

                let (sql, values) = feed.into_insert().build_postgres(PostgresQueryBuilder);

                let stmt = tx.prepare_cached(&sql).await?;
                let row = tx.query_one(&stmt, &values.as_params()).await?;

                let subscription_id = {
                    let (sql, values) = SubscriptionInsert {
                        id: Uuid::new_v4(),
                        title: &title,
                        feed_id: row.get("id"),
                        user_id: &data.user_id,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        upsert: true,
                    }
                    .into_insert()
                    .build_postgres(PostgresQueryBuilder);

                    let stmt = tx.prepare_cached(&sql).await?;
                    let row = tx.query_one(&stmt, &values.as_params()).await?;

                    row.get("id")
                };

                if let Some(tag_id) = parent_id {
                    let subscription_tag = SubscriptionTagInsert {
                        subscription_id,
                        user_id: &data.user_id,
                        tag_ids: vec![tag_id],
                    };

                    let (sql, values) = subscription_tag
                        .into_insert()
                        .build_postgres(PostgresQueryBuilder);

                    let stmt = tx.prepare_cached(&sql).await?;
                    tx.execute(&stmt, &values.as_params()).await?;
                }
            }
        }

        tx.commit().await?;

        Ok(())
    }
}

struct SubscriptionTagRow {
    subscription_id: Uuid,
    id: Uuid,
    title: String,
    user_id: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<&Row> for SubscriptionTagRow {
    fn from(value: &Row) -> Self {
        Self {
            subscription_id: value.get("subscription_id"),
            id: value.get("id"),
            title: value.get("title"),
            user_id: value.get("user_id"),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
        }
    }
}

impl From<SubscriptionTagRow> for Tag {
    fn from(value: SubscriptionTagRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            bookmark_count: None,
            feed_count: None,
        }
    }
}

struct SubscriptionRow<'a>(&'a Row);

impl From<SubscriptionRow<'_>> for Subscription {
    fn from(SubscriptionRow(value): SubscriptionRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            title: value.get("title"),
            feed_id: value.get("feed_id"),
            user_id: value.get("user_id"),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
            feed: value.try_get::<_, String>("link").ok().map(|link| Feed {
                id: value.get("feed_id"),
                link: link.parse().unwrap(),
                xml_url: value
                    .get::<_, Option<String>>("xml_url")
                    .and_then(|e| e.parse().ok()),
                title: value.get("feed_title"),
                description: value.get("description"),
                refreshed_at: value.get("refreshed_at"),
                entries: None,
            }),
            tags: None,
            unread_count: None,
        }
    }
}
