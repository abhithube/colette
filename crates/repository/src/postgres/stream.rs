use chrono::{DateTime, Utc};
use colette_core::{
    Stream,
    stream::{Error, StreamParams, StreamRepository},
    subscription_entry::SubscriptionEntryFilter,
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    stream::{StreamDelete, StreamInsert, StreamSelect},
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder as _;
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresStreamRepository {
    pool: PgPool,
}

impl PostgresStreamRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl StreamRepository for PostgresStreamRepository {
    async fn query(&self, params: StreamParams) -> Result<Vec<Stream>, Error> {
        let (sql, values) = StreamSelect {
            id: params.id,
            user_id: params.user_id,
            cursor: params.cursor.as_deref(),
            limit: params.limit.map(|e| e as u64),
        }
        .into_select()
        .build_sqlx(PostgresQueryBuilder);
        let rows = sqlx::query_as_with::<_, StreamRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn save(&self, data: &Stream) -> Result<(), Error> {
        let (sql, values) = StreamInsert {
            id: data.id,
            title: &data.title,
            filter: serde_json::to_value(&data.filter).unwrap(),
            user_id: data.user_id,
            created_at: data.created_at,
            updated_at: data.updated_at,
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
        let (sql, values) = StreamDelete { id }
            .into_delete()
            .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct StreamRow {
    id: Uuid,
    title: String,
    filter_json: Json<SubscriptionEntryFilter>,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<StreamRow> for Stream {
    fn from(value: StreamRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            filter: value.filter_json.0,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
