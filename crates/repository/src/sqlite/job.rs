use colette_core::job::{Error, Job, JobParams, JobRepository};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    job::{JobDelete, JobInsert},
};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder as _;
use uuid::Uuid;

use super::{PreparedClient as _, SqliteRow};

#[derive(Debug, Clone)]
pub struct SqliteJobRepository {
    pool: Pool,
}

impl SqliteJobRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl JobRepository for SqliteJobRepository {
    async fn query(&self, params: JobParams) -> Result<Vec<Job>, Error> {
        let client = self.pool.get().await?;

        let jobs = client
            .interact(move |conn| {
                let (sql, values) = params.into_select().build_rusqlite(SqliteQueryBuilder);
                conn.query_prepared::<Job>(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(jobs)
    }

    async fn save(&self, data: &Job) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let data = data.to_owned();

        client
            .interact(move |conn| {
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
                .build_rusqlite(SqliteQueryBuilder);
                conn.execute_prepared(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        client
            .interact(move |conn| {
                let (sql, values) = JobDelete { id }
                    .into_delete()
                    .build_rusqlite(SqliteQueryBuilder);
                conn.execute_prepared(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(())
    }
}

impl From<SqliteRow<'_>> for Job {
    fn from(SqliteRow(value): SqliteRow<'_>) -> Self {
        Self {
            id: value.get_unwrap("id"),
            job_type: value.get_unwrap("job_type"),
            data: serde_json::from_value(value.get_unwrap("data_json")).unwrap(),
            status: value.get_unwrap::<_, String>("status").parse().unwrap(),
            group_identifier: value.get_unwrap("group_identifier"),
            message: value.get_unwrap("message"),
            created_at: value.get_unwrap("created_at"),
            completed_at: value.get_unwrap("completed_at"),
        }
    }
}
