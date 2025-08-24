use chrono::{DateTime, Utc};
use colette_authentication::UserId;
use colette_common::RepositoryError;
use colette_crud::{
    Entry, EntryBooleanField, EntryDateField, EntryFilter, EntryId, EntryRepository,
    EntryTextField, ReadStatus,
};
use colette_handler::{EntryDto, EntryQueryParams, EntryQueryRepository};
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use crate::{DbUrl, ToColumn, ToSql};

const BASE_QUERY: &str = include_str!("../queries/entries/find.sql");

#[derive(Debug, Clone)]
pub struct PostgresEntryRepository {
    pool: PgPool,
}

impl PostgresEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EntryRepository for PostgresEntryRepository {
    async fn find_by_id(
        &self,
        id: EntryId,
        user_id: UserId,
    ) -> Result<Option<Entry>, RepositoryError> {
        let entry = sqlx::query_file_as!(
            EntryByIdRow,
            "queries/entries/find_by_id.sql",
            id.as_inner(),
            user_id.as_inner()
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entry)
    }

    async fn save(&self, data: &Entry) -> Result<(), RepositoryError> {
        match data.read_status() {
            ReadStatus::Unread => {
                sqlx::query_file!(
                    "queries/read_statuses/delete_by_id.sql",
                    data.id().as_inner(),
                    data.user_id().as_inner()
                )
                .execute(&self.pool)
                .await?;
            }
            ReadStatus::Read(read_at) => {
                sqlx::query_file!(
                    "queries/read_statuses/insert.sql",
                    data.id().as_inner(),
                    data.user_id().as_inner(),
                    read_at
                )
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct EntryByIdRow {
    id: Uuid,
    read_at: Option<DateTime<Utc>>,
    user_id: Uuid,
}

impl From<EntryByIdRow> for Entry {
    fn from(value: EntryByIdRow) -> Self {
        Self::from_unchecked(
            value.id,
            if let Some(read_at) = value.read_at {
                ReadStatus::Read(read_at)
            } else {
                ReadStatus::Unread
            },
            value.user_id,
        )
    }
}

#[async_trait::async_trait]
impl EntryQueryRepository for PostgresEntryRepository {
    async fn query(&self, params: EntryQueryParams) -> Result<Vec<EntryDto>, RepositoryError> {
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
            .build_query_as::<EntryRow>()
            .bind(params.user_id)
            .bind(params.id)
            .bind(params.subscription_id)
            .bind(params.has_read)
            .bind(params.tags)
            .bind(cursor_published_at)
            .bind(cursor_id)
            .bind(params.limit.map(|e| e as i64))
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[derive(sqlx::FromRow)]
struct EntryRow {
    id: Uuid,
    link: DbUrl,
    title: String,
    published_at: DateTime<Utc>,
    description: Option<String>,
    author: Option<String>,
    thumbnail_url: Option<DbUrl>,
    read_at: Option<DateTime<Utc>>,
    feed_id: Uuid,
}

impl From<EntryRow> for EntryDto {
    fn from(value: EntryRow) -> Self {
        Self {
            id: value.id,
            link: value.link.into(),
            title: value.title,
            published_at: value.published_at,
            description: value.description,
            thumbnail_url: value.thumbnail_url.map(Into::into),
            author: value.author,
            read_status: if let Some(read_at) = value.read_at {
                ReadStatus::Read(read_at)
            } else {
                ReadStatus::Unread
            },
            feed_id: value.feed_id,
        }
    }
}

impl ToColumn for EntryTextField {
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

impl ToColumn for EntryBooleanField {
    fn to_column(self) -> String {
        match self {
            Self::HasRead => "has_read".into(),
        }
    }
}

impl ToColumn for EntryDateField {
    fn to_column(self) -> String {
        match self {
            Self::PublishedAt => "published_at".into(),
        }
    }
}

impl ToSql for EntryFilter {
    fn to_sql(self) -> String {
        match self {
            EntryFilter::Text { field, op } => match field {
                EntryTextField::Tag => format!(
                    "EXISTS (SELECT 1 FROM subscriptions_tags st INNER JOIN tags t ON t.id = st.tag_id WHERE st.subscription_id = s.id AND {})",
                    (field.to_column().as_str(), op).to_sql()
                ),
                _ => (field.to_column().as_str(), op).to_sql(),
            },
            EntryFilter::Boolean { field, op } => (field.to_column().as_str(), op).to_sql(),
            EntryFilter::Date { field, op } => (field.to_column().as_str(), op).to_sql(),
            EntryFilter::And(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut and = conditions.swap_remove(0);

                for condition in conditions {
                    and = format!("{and} AND {condition}");
                }

                and
            }
            EntryFilter::Or(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut or = conditions.swap_remove(0);

                for condition in conditions {
                    or = format!("{or} OR {condition}");
                }

                or
            }
            EntryFilter::Not(filter) => format!("NOT {}", (*filter).to_sql()),
        }
    }
}

#[allow(dead_code)]
fn validate_base_query() {
    let _ = sqlx::query_file!(
        "queries/entries/find.sql",
        Uuid::now_v7(),
        Option::<Uuid>::None,
        Option::<Uuid>::None,
        Option::<bool>::None,
        Option::<&[Uuid]>::None,
        Option::<DateTime<Utc>>::None,
        Option::<Uuid>::None,
        1
    );
}
