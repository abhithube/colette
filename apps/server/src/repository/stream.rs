use colette_core::{
    Stream,
    common::Transaction,
    stream::{
        Error, StreamById, StreamCreateParams, StreamDeleteParams, StreamFindByIdParams,
        StreamFindParams, StreamRepository, StreamUpdateParams,
    },
};
use colette_model::StreamRow;
use colette_query::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, FromQueryResult};

#[derive(Debug, Clone)]
pub struct SqliteStreamRepository {
    db: DatabaseConnection,
}

impl SqliteStreamRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl StreamRepository for SqliteStreamRepository {
    async fn find_streams(&self, params: StreamFindParams) -> Result<Vec<Stream>, Error> {
        let streams = StreamRow::find_by_statement(
            self.db.get_database_backend().build(&params.into_select()),
        )
        .all(&self.db)
        .await
        .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(streams)
    }

    async fn find_stream_by_id(
        &self,
        tx: &dyn Transaction,
        params: StreamFindByIdParams,
    ) -> Result<StreamById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let id = params.id;

        let Some(result) = tx
            .query_one(self.db.get_database_backend().build(&params.into_select()))
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(StreamById {
            id: result
                .try_get_by_index::<String>(0)
                .unwrap()
                .parse()
                .unwrap(),
            user_id: result
                .try_get_by_index::<String>(1)
                .unwrap()
                .parse()
                .unwrap(),
        })
    }

    async fn create_stream(&self, params: StreamCreateParams) -> Result<(), Error> {
        let title = params.title.clone();

        self.db
            .execute(self.db.get_database_backend().build(&params.into_insert()))
            .await
            .map_err(|e| match e {
                DbErr::RecordNotInserted => Error::Conflict(title),
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn update_stream(
        &self,
        tx: &dyn Transaction,
        params: StreamUpdateParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        if params.title.is_none() && params.filter.is_none() {
            return Ok(());
        }

        tx.execute(self.db.get_database_backend().build(&params.into_update()))
            .await?;

        Ok(())
    }

    async fn delete_stream(
        &self,
        tx: &dyn Transaction,
        params: StreamDeleteParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        tx.execute(self.db.get_database_backend().build(&params.into_delete()))
            .await?;

        Ok(())
    }
}
