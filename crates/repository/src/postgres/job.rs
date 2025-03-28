use colette_core::job::{Error, Job, JobParams, JobRepository};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    job::{JobDelete, JobInsert},
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresJobRepository {
    pool: Pool,
}

impl PostgresJobRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl JobRepository for PostgresJobRepository {
    async fn query(&self, params: JobParams) -> Result<Vec<Job>, Error> {
        let client = self.pool.get().await?;

        let (sql, values) = params.into_select().build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;

        Ok(rows.iter().map(|e| JobRow(e).into()).collect())
    }

    async fn save(&self, data: &Job) -> Result<(), Error> {
        let client = self.pool.get().await?;

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
        .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        client.execute(&stmt, &values.as_params()).await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = JobDelete { id }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        client.execute(&stmt, &values.as_params()).await?;

        Ok(())
    }
}

struct JobRow<'a>(&'a Row);

impl From<JobRow<'_>> for Job {
    fn from(JobRow(value): JobRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            job_type: value.get("job_type"),
            data: serde_json::from_value(value.get("data_json")).unwrap(),
            status: value.get::<_, String>("status").parse().unwrap(),
            group_identifier: value.get("group_identifier"),
            message: value.get("message"),
            created_at: value.get("created_at"),
            completed_at: value.get("completed_at"),
        }
    }
}
