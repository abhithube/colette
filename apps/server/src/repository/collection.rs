use colette_core::{
    Collection,
    collection::{
        CollectionById, CollectionCreateParams, CollectionDeleteParams, CollectionFindByIdParams,
        CollectionFindParams, CollectionRepository, CollectionUpdateParams, Error,
    },
    common::Transaction,
};
use colette_model::CollectionRow;
use colette_query::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, FromQueryResult};

#[derive(Debug, Clone)]
pub struct SqliteCollectionRepository {
    db: DatabaseConnection,
}

impl SqliteCollectionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl CollectionRepository for SqliteCollectionRepository {
    async fn find_collections(
        &self,
        params: CollectionFindParams,
    ) -> Result<Vec<Collection>, Error> {
        let collections = CollectionRow::find_by_statement(
            self.db.get_database_backend().build(&params.into_select()),
        )
        .all(&self.db)
        .await
        .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(collections)
    }

    async fn find_collection_by_id(
        &self,
        tx: &dyn Transaction,
        params: CollectionFindByIdParams,
    ) -> Result<CollectionById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let id = params.id;

        let Some(result) = tx
            .query_one(self.db.get_database_backend().build(&params.into_select()))
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(CollectionById {
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

    async fn create_collection(&self, params: CollectionCreateParams) -> Result<(), Error> {
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

    async fn update_collection(
        &self,
        tx: &dyn Transaction,
        params: CollectionUpdateParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        if params.title.is_none() && params.filter.is_none() {
            return Ok(());
        }

        tx.execute(self.db.get_database_backend().build(&params.into_update()))
            .await?;

        Ok(())
    }

    async fn delete_collection(
        &self,
        tx: &dyn Transaction,
        params: CollectionDeleteParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        tx.execute(self.db.get_database_backend().build(&params.into_delete()))
            .await?;

        Ok(())
    }
}
