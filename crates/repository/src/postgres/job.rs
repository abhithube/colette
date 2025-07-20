use chrono::{DateTime, Utc};
use colette_core::job::{Error, Job, JobParams, JobRepository};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    job::{JobDelete, JobInsert, JobSelect},
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder as _;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresJobRepository {
    pool: PgPool,
}

impl PostgresJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl JobRepository for PostgresJobRepository {
    async fn query(&self, params: JobParams) -> Result<Vec<Job>, Error> {
        let (sql, values) = JobSelect {
            id: params.id,
            group_identifier: params.group_identifier.as_deref(),
        }
        .into_select()
        .build_sqlx(PostgresQueryBuilder);
        let rows = sqlx::query_as_with::<_, JobRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn save(&self, data: &Job) -> Result<(), Error> {
        let (sql, values) = JobInsert {
            id: data.id,
            job_type: &data.job_type,
            data: serde_json::to_value(&data.data).unwrap(),
            status: &data.status.to_string(),
            group_identifier: data.group_identifier.as_deref(),
            message: data.message.as_deref(),
            created_at: data.created_at,
            completed_at: data.completed_at,
        }
        .into_insert()
        .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = JobDelete { id }
            .into_delete()
            .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).execute(&self.pool).await?;

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct JobRow {
    pub id: Uuid,
    pub job_type: String,
    pub data_json: Value,
    pub status: String,
    pub group_identifier: Option<String>,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<JobRow> for Job {
    fn from(value: JobRow) -> Self {
        Self {
            id: value.id,
            job_type: value.job_type,
            data: value.data_json,
            status: value.status.parse().unwrap(),
            group_identifier: value.group_identifier,
            message: value.message,
            created_at: value.created_at,
            completed_at: value.completed_at,
        }
    }
}
