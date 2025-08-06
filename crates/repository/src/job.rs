use chrono::{DateTime, Utc};
use colette_core::job::{
    Error, Job, JobById, JobFindParams, JobInsertParams, JobRepository, JobStatus, JobUpdateParams,
};
use serde_json::Value;
use sqlx::{
    Decode, Encode, PgPool, Postgres, Type,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef},
    types::Json,
};
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
    async fn find(&self, params: JobFindParams) -> Result<Vec<Job>, Error> {
        let jobs = sqlx::query_file_as!(
            JobRow,
            "queries/jobs/find.sql",
            params.id,
            params.group_identifier,
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(jobs)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<JobById>, Error> {
        let job = sqlx::query_file_as!(JobByIdRow, "queries/jobs/find_by_id.sql", id)
            .map(Into::into)
            .fetch_optional(&self.pool)
            .await?;

        Ok(job)
    }

    async fn insert(&self, params: JobInsertParams) -> Result<Uuid, Error> {
        let id = sqlx::query_file_scalar!(
            "queries/jobs/insert.sql",
            params.job_type,
            Json(params.data) as Json<Value>,
            params.group_identifier
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    async fn update(&self, params: JobUpdateParams) -> Result<(), Error> {
        let (has_message, message) = if let Some(message) = params.message {
            (true, message)
        } else {
            (false, None)
        };

        sqlx::query_file_scalar!(
            "queries/jobs/update.sql",
            params.id,
            params.status.map(DbJobStatus) as Option<DbJobStatus>,
            has_message,
            message
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query_file!("queries/jobs/delete_by_id.sql", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

struct JobRow {
    id: Uuid,
    job_type: String,
    data_json: Json<Value>,
    status: DbJobStatus,
    group_identifier: Option<String>,
    message: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<JobRow> for Job {
    fn from(value: JobRow) -> Self {
        Self {
            id: value.id,
            job_type: value.job_type,
            data: value.data_json.0,
            status: value.status.into(),
            group_identifier: value.group_identifier,
            message: value.message,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

struct JobByIdRow {
    id: Uuid,
    status: DbJobStatus,
}

impl From<JobByIdRow> for JobById {
    fn from(value: JobByIdRow) -> Self {
        Self {
            id: value.id,
            status: value.status.into(),
        }
    }
}

struct DbJobStatus(JobStatus);

impl From<DbJobStatus> for JobStatus {
    fn from(value: DbJobStatus) -> Self {
        value.0
    }
}

impl From<JobStatus> for DbJobStatus {
    fn from(value: JobStatus) -> Self {
        Self(value)
    }
}

impl Type<Postgres> for DbJobStatus {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("text")
    }
}

impl Encode<'_, Postgres> for DbJobStatus {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.extend_from_slice(self.0.to_string().as_bytes());

        Ok(IsNull::No)
    }
}

impl Decode<'_, Postgres> for DbJobStatus {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => str::from_utf8(value.as_bytes()?)?.parse(),
            PgValueFormat::Text => value.as_str()?.parse(),
        }
        .map(DbJobStatus)
        .map_err(Into::into)
    }
}
