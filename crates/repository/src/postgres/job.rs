use colette_core::job::{Error, Job, JobParams, JobRepository};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    job::{JobDelete, JobInsert, JobSelect},
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder as _;
use uuid::Uuid;

use super::{PgRow, PreparedClient as _};

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

        let (sql, values) = JobSelect {
            id: params.id,
            group_identifier: params.group_identifier.as_deref(),
        }
        .into_select()
        .build_postgres(PostgresQueryBuilder);
        let jobs = client.query_prepared::<Job>(&sql, &values).await?;

        Ok(jobs)
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

        client.execute_prepared(&sql, &values).await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = JobDelete { id }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

        client.execute_prepared(&sql, &values).await?;

        Ok(())
    }
}

impl From<PgRow<'_>> for Job {
    fn from(PgRow(value): PgRow<'_>) -> Self {
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
