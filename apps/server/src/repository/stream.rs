use chrono::{DateTime, Utc};
use colette_core::{
    FeedEntry, Stream,
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    stream::{
        Error, StreamCreateData, StreamEntryFindParams, StreamFindParams, StreamRepository,
        StreamUpdateData,
    },
};
use serde_json::Value;
use sqlx::{Pool, Postgres, QueryBuilder, types::Json};
use uuid::Uuid;

use super::{common::ToSql, feed_entry::FeedEntryRow};

#[derive(Debug, Clone)]
pub struct PostgresStreamRepository {
    pool: Pool<Postgres>,
}

impl PostgresStreamRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresStreamRepository {
    type Params = StreamFindParams;
    type Output = Result<Vec<Stream>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let streams = sqlx::query_file_as!(
            StreamRow,
            "queries/streams/select.sql",
            params.user_id,
            params.id.is_none(),
            params.id,
            params.cursor.is_none(),
            params.cursor.map(|e| e.title),
            params.limit
        )
        .fetch_all(&self.pool)
        .await
        .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(streams)
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresStreamRepository {
    type Data = StreamCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        sqlx::query_file_scalar!(
            "queries/streams/insert.sql",
            data.title,
            serde_json::to_value(data.filter).unwrap(),
            data.user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
            _ => Error::Database(e),
        })
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresStreamRepository {
    type Params = IdParams;
    type Data = StreamUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() {
            sqlx::query_file!(
                "queries/streams/update.sql",
                params.id,
                params.user_id,
                data.title.is_some(),
                data.title,
                data.filter.is_some(),
                data.filter.map(|e| serde_json::to_value(e).unwrap())
            )
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Database(e),
            })?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresStreamRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let result = sqlx::query_file!("queries/streams/delete.sql", params.id, params.user_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl StreamRepository for PostgresStreamRepository {
    async fn find_entries(&self, params: StreamEntryFindParams) -> Result<Vec<FeedEntry>, Error> {
        let initial = format!(
            r#"SELECT ufe.id,
       fe.link,
       fe.title,
       fe.published_at,
       fe.description,
       fe.author,
       fe.thumbnail_url,
       ufe.has_read,
       ufe.user_feed_id AS feed_id,
       ufe.created_at,
       ufe.updated_at
  FROM user_feed_entries ufe
  JOIN feed_entries fe on fe.id = ufe.feed_entry_id
 WHERE ufe.user_id = '{}'"#,
            params.user_id
        );

        let mut qb = QueryBuilder::new(initial);

        let where_clause = params.filter.to_sql();
        if !where_clause.is_empty() {
            qb.push(" AND ");
            qb.push(&where_clause);
        }

        if let Some(cursor) = params.cursor {
            qb.push(" AND (fe.published_at, ufe.id) > (");

            let mut separated = qb.separated(", ");
            separated.push_bind(cursor.published_at);
            separated.push_bind(cursor.id);
            separated.push_unseparated(")");
        }

        qb.push("\n ORDER BY fe.published_at DESC, ufe.id DESC");

        if let Some(limit) = params.limit {
            qb.push("\n LIMIT ");
            qb.push_bind(limit);
        }

        let query = qb.build_query_as::<FeedEntryRow>();

        let entries = query
            .fetch_all(&self.pool)
            .await
            .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(entries)
    }
}

struct StreamRow {
    id: Uuid,
    title: String,
    filter: Json<Value>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

impl From<StreamRow> for Stream {
    fn from(value: StreamRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            filter: serde_json::from_value(value.filter.0).unwrap(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
