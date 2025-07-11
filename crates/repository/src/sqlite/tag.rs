use colette_core::{
    Tag,
    tag::{Error, TagParams, TagRepository, TagType},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    tag::{TagBase, TagDelete, TagInsert, TagSelect},
};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder as _;
use uuid::Uuid;

use super::{PreparedClient as _, SqliteRow};

#[derive(Debug, Clone)]
pub struct SqliteTagRepository {
    pool: Pool,
}

impl SqliteTagRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TagRepository for SqliteTagRepository {
    async fn query(&self, params: TagParams) -> Result<Vec<Tag>, Error> {
        let client = self.pool.get().await?;

        let tags = client
            .interact(move |conn| {
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
                .build_rusqlite(SqliteQueryBuilder);
                conn.query_prepared::<Tag>(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(tags)
    }

    async fn save(&self, data: &Tag) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let data = data.to_owned();

        client
            .interact(move |conn| {
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
                .build_rusqlite(SqliteQueryBuilder);
                conn.execute_prepared(&sql, &values).map_err(|e| {
                    match e.sqlite_error().map(|e| e.extended_code) {
                        Some(rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE) => {
                            Error::Conflict(data.title.clone())
                        }
                        _ => Error::SqliteClient(e),
                    }
                })
            })
            .await
            .unwrap()?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        client
            .interact(move |conn| {
                let (sql, values) = TagDelete {
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
}

impl From<SqliteRow<'_>> for Tag {
    fn from(SqliteRow(value): SqliteRow<'_>) -> Self {
        Self {
            id: value.get_unwrap("id"),
            title: value.get_unwrap("title"),
            user_id: value.get_unwrap("user_id"),
            created_at: value.get_unwrap("created_at"),
            updated_at: value.get_unwrap("updated_at"),
            subscription_count: value.get("subscription_count").ok(),
            bookmark_count: value.get("bookmark_count").ok(),
        }
    }
}
