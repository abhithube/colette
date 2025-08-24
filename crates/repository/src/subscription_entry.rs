use chrono::{DateTime, Utc};
use colette_core::{
    FeedEntry, SubscriptionEntry,
    common::RepositoryError,
    subscription_entry::{
        SubscriptionEntryBooleanField, SubscriptionEntryDateField, SubscriptionEntryFilter,
        SubscriptionEntryFindParams, SubscriptionEntryId, SubscriptionEntryRepository,
        SubscriptionEntryTextField,
    },
};
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use crate::{DbUrl, ToColumn, ToSql};

const BASE_QUERY: &str = include_str!("../queries/subscription_entries/find.sql");

#[allow(dead_code)]
fn validate_base_query() {
    let _ = sqlx::query_file!(
        "queries/subscription_entries/find.sql",
        Option::<Uuid>::None,
        Option::<Uuid>::None,
        Option::<Uuid>::None,
        Option::<bool>::None,
        Option::<&[Uuid]>::None,
        Option::<DateTime<Utc>>::None,
        Option::<Uuid>::None,
        1
    );
}

#[derive(Debug, Clone)]
pub struct PostgresSubscriptionEntryRepository {
    pool: PgPool,
}

impl PostgresSubscriptionEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionEntryRepository for PostgresSubscriptionEntryRepository {
    async fn find(
        &self,
        params: SubscriptionEntryFindParams,
    ) -> Result<Vec<SubscriptionEntry>, RepositoryError> {
        let (cursor_published_at, cursor_id) = if let Some((published_at, id)) = params.cursor {
            (Some(published_at), Some(id))
        } else {
            (None, None)
        };

        let mut qb = QueryBuilder::new(format!(
            r#"WITH results AS ({BASE_QUERY}) SELECT * FROM results WHERE TRUE"#
        ));

        if let Some(filter) = params.filter {
            qb.push(format!(" {}", filter.to_sql()));
        }

        let rows = qb
            .build_query_as::<SubscriptionEntryRow>()
            .bind(params.id.map(|e| e.as_inner()))
            .bind(params.user_id.map(|e| e.as_inner()))
            .bind(params.subscription_id.map(|e| e.as_inner()))
            .bind(params.has_read)
            .bind(
                params
                    .tags
                    .map(|e| e.iter().map(|e| e.as_inner()).collect::<Vec<_>>()),
            )
            .bind(cursor_published_at)
            .bind(cursor_id)
            .bind(params.limit.map(|e| e as i64))
            .bind(params.with_feed_entry)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn mark_as_read(&self, id: SubscriptionEntryId) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/subscription_entries/mark_as_read.sql",
            id.as_inner()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_as_unread(&self, id: SubscriptionEntryId) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/subscription_entries/mark_as_unread.sql",
            id.as_inner()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct SubscriptionEntryRow {
    id: Uuid,
    has_read: bool,
    read_at: Option<DateTime<Utc>>,
    subscription_id: Uuid,
    feed_entry_id: Uuid,
    link: DbUrl,
    title: String,
    published_at: DateTime<Utc>,
    description: Option<String>,
    author: Option<String>,
    thumbnail_url: Option<DbUrl>,
    feed_id: Uuid,
    user_id: Uuid,
}

impl From<SubscriptionEntryRow> for SubscriptionEntry {
    fn from(value: SubscriptionEntryRow) -> Self {
        Self {
            id: value.id.into(),
            has_read: value.has_read,
            read_at: value.read_at,
            subscription_id: value.subscription_id.into(),
            feed_entry_id: value.id.into(),
            user_id: value.user_id.into(),
            feed_entry: FeedEntry {
                id: value.feed_entry_id.into(),
                link: value.link.into(),
                title: value.title,
                published_at: value.published_at,
                description: value.description,
                thumbnail_url: value.thumbnail_url.map(Into::into),
                author: value.author,
                feed_id: value.feed_id.into(),
            },
        }
    }
}

impl ToColumn for SubscriptionEntryTextField {
    fn to_column(self) -> String {
        match self {
            Self::Link => "link".into(),
            Self::Title => "title".into(),
            Self::Description => "description".into(),
            Self::Author => "author".into(),
            Self::Tag => "t.title".into(),
        }
    }
}

impl ToColumn for SubscriptionEntryBooleanField {
    fn to_column(self) -> String {
        match self {
            Self::HasRead => "has_read".into(),
        }
    }
}

impl ToColumn for SubscriptionEntryDateField {
    fn to_column(self) -> String {
        match self {
            Self::PublishedAt => "published_at".into(),
        }
    }
}

impl ToSql for SubscriptionEntryFilter {
    fn to_sql(self) -> String {
        match self {
            SubscriptionEntryFilter::Text { field, op } => match field {
                SubscriptionEntryTextField::Tag => format!(
                    "EXISTS (SELECT 1 FROM subscriptions_tags st INNER JOIN tags t ON t.id = st.tag_id WHERE st.subscription_id = s.id AND {})",
                    (field.to_column().as_str(), op).to_sql()
                ),
                _ => (field.to_column().as_str(), op).to_sql(),
            },
            SubscriptionEntryFilter::Boolean { field, op } => {
                (field.to_column().as_str(), op).to_sql()
            }
            SubscriptionEntryFilter::Date { field, op } => {
                (field.to_column().as_str(), op).to_sql()
            }
            SubscriptionEntryFilter::And(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut and = conditions.swap_remove(0);

                for condition in conditions {
                    and = format!("{and} AND {condition}");
                }

                and
            }
            SubscriptionEntryFilter::Or(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut or = conditions.swap_remove(0);

                for condition in conditions {
                    or = format!("{or} OR {condition}");
                }

                or
            }
            SubscriptionEntryFilter::Not(filter) => format!("NOT {}", (*filter).to_sql()),
        }
    }
}
