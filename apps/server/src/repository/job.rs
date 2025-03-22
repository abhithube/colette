use chrono::{DateTime, Utc};
use colette_core::job::{Error, Job, JobFindParams, JobRepository};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    job::{JobDelete, JobInsert, JobSelect, JobSelectOne},
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use serde_json::Value;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteJobRepository {
    pool: Pool<Sqlite>,
}

impl SqliteJobRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl JobRepository for SqliteJobRepository {
    async fn find(&self, params: JobFindParams) -> Result<Vec<Job>, Error> {
        let (sql, values) = JobSelect {
            id: params.id,
            group_id: params.group_id.as_deref(),
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, JobRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Job>, Error> {
        let (sql, values) = JobSelectOne { id }
            .into_select()
            .build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, JobRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(Into::into))
    }

    async fn save(&self, data: &Job, upsert: bool) -> Result<(), Error> {
        let (sql, values) = JobInsert {
            id: data.id,
            job_type: &data.job_type,
            data: &serde_json::to_string(&data.data).unwrap(),
            status: &data.status.to_string(),
            group_id: data.group_id.as_deref(),
            message: data.message.as_deref(),
            completed_at: data.completed_at,
            upsert,
        }
        .into_insert()
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = JobDelete { id }
            .into_delete()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct JobRow {
    pub id: Uuid,
    pub job_type: String,
    pub data: Value,
    pub status: String,
    pub group_id: Option<String>,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<JobRow> for Job {
    fn from(value: JobRow) -> Self {
        Self {
            id: value.id,
            job_type: value.job_type,
            data: value.data,
            status: value.status.parse().unwrap(),
            group_id: value.group_id,
            message: value.message,
            created_at: value.created_at,
            completed_at: value.completed_at,
        }
    }
}
