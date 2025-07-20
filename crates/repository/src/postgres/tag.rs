use chrono::{DateTime, Utc};
use colette_core::{
    Tag,
    tag::{Error, TagParams, TagRepository, TagType},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    tag::{TagBase, TagDelete, TagInsert, TagSelect},
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder as _;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresTagRepository {
    pool: PgPool,
}

impl PostgresTagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TagRepository for PostgresTagRepository {
    async fn query(&self, params: TagParams) -> Result<Vec<Tag>, Error> {
        let (sql, values) = TagSelect {
            ids: params.ids,
            tag_type: params.tag_type.map(|e| match e {
                TagType::Bookmarks => colette_query::tag::TagType::Bookmarks,
                TagType::Feeds => colette_query::tag::TagType::Feeds,
            }),
            feed_id: params.feed_id,
            bookmark_id: params.bookmark_id,
            user_id: params.user_id,
            cursor: params.cursor.as_deref(),
            limit: params.limit.map(|e| e as u64),
            with_subscription_count: params.with_subscription_count,
            with_bookmark_count: params.with_bookmark_count,
            ..Default::default()
        }
        .into_select()
        .build_sqlx(PostgresQueryBuilder);
        let rows = sqlx::query_as_with::<_, TagRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn save(&self, data: &Tag) -> Result<(), Error> {
        let (sql, values) = TagInsert {
            tags: [TagBase {
                id: data.id,
                title: &data.title,
                created_at: data.created_at,
                updated_at: data.updated_at,
            }],
            user_id: data.user_id,
            upsert: false,
        }
        .into_insert()
        .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => {
                    Error::Conflict(data.title.clone())
                }
                _ => Error::Sqlx(e),
            })?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = TagDelete {
            id: Some(id),
            ..Default::default()
        }
        .into_delete()
        .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct TagRow {
    pub(crate) id: Uuid,
    pub(crate) title: String,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[sqlx(default)]
    subscription_count: Option<i64>,
    #[sqlx(default)]
    bookmark_count: Option<i64>,
}

impl From<TagRow> for Tag {
    fn from(value: TagRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            subscription_count: value.subscription_count,
            bookmark_count: value.bookmark_count,
        }
    }
}
