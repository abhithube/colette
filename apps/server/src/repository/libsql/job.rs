use chrono::{DateTime, Utc};
use colette_core::job::{Error, Job, JobParams, JobRepository};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    job::{JobDelete, JobInsert, JobSelect, JobSelectOne},
};
use libsql::Connection;
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;

use super::LibsqlBinder;

#[derive(Debug, Clone)]
pub struct LibsqlJobRepository {
    conn: Connection,
}

impl LibsqlJobRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl JobRepository for LibsqlJobRepository {
    async fn query(&self, params: JobParams) -> Result<Vec<Job>, Error> {
        let (sql, values) = JobSelect {
            id: params.id,
            group_id: params.group_id.as_deref(),
        }
        .into_select()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let mut jobs = Vec::<Job>::new();
        while let Some(row) = rows.next().await? {
            jobs.push(libsql::de::from_row::<JobRow>(&row)?.into());
        }

        Ok(jobs)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Job>, Error> {
        let (sql, values) = JobSelectOne { id }
            .into_select()
            .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(libsql::de::from_row::<JobRow>(&row)?.into()))
    }

    async fn save(&self, data: &Job) -> Result<(), Error> {
        let (sql, values) = JobInsert {
            id: data.id,
            job_type: &data.job_type,
            data: &serde_json::to_string(&data.data).unwrap(),
            status: &data.status.to_string(),
            group_id: data.group_id.as_deref(),
            message: data.message.as_deref(),
            created_at: data.created_at,
            completed_at: data.completed_at,
        }
        .into_insert()
        .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        stmt.execute(values.into_params()).await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let (sql, values) = JobDelete { id }
            .into_delete()
            .build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        stmt.execute(values.into_params()).await?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct JobRow {
    pub id: Uuid,
    pub job_type: String,
    pub data: String,
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
            data: serde_json::from_str(&value.data).unwrap(),
            status: value.status.parse().unwrap(),
            group_id: value.group_id,
            message: value.message,
            created_at: value.created_at,
            completed_at: value.completed_at,
        }
    }
}
