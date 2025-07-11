use colette_core::{
    Tag,
    tag::{Error, TagParams, TagRepository, TagType},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    tag::{TagBase, TagDelete, TagInsert, TagSelect},
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder as _;
use tokio_postgres::error::SqlState;
use uuid::Uuid;

use super::{PgRow, PreparedClient as _};

#[derive(Debug, Clone)]
pub struct PostgresTagRepository {
    pool: Pool,
}

impl PostgresTagRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TagRepository for PostgresTagRepository {
    async fn query(&self, params: TagParams) -> Result<Vec<Tag>, Error> {
        let client = self.pool.get().await?;

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
        .build_postgres(PostgresQueryBuilder);
        let tags = client.query_prepared::<Tag>(&sql, &values).await?;

        Ok(tags)
    }

    async fn save(&self, data: &Tag) -> Result<(), Error> {
        let client = self.pool.get().await?;

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
        .build_postgres(PostgresQueryBuilder);

        client
            .execute_prepared(&sql, &values)
            .await
            .map_err(|e| match e.code() {
                Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.title.clone()),
                _ => Error::PostgresClient(e),
            })?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = TagDelete {
            id: Some(id),
            ..Default::default()
        }
        .into_delete()
        .build_postgres(PostgresQueryBuilder);

        client.execute_prepared(&sql, &values).await?;

        Ok(())
    }
}

impl From<PgRow<'_>> for Tag {
    fn from(PgRow(value): PgRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            title: value.get("title"),
            user_id: value.get("user_id"),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
            subscription_count: value.try_get("subscription_count").ok(),
            bookmark_count: value.try_get("bookmark_count").ok(),
        }
    }
}
