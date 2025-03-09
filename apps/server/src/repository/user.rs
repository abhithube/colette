use colette_core::{
    User,
    common::Transaction,
    user::{Error, UserCreateParams, UserFindParams, UserRepository},
};
use colette_model::UserRow;
use colette_query::{IntoInsert, IntoSelect};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, FromQueryResult};

#[derive(Debug, Clone)]
pub struct SqliteUserRepository {
    db: DatabaseConnection,
}

impl SqliteUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl UserRepository for SqliteUserRepository {
    async fn find_user(&self, params: UserFindParams) -> Result<User, Error> {
        let id = params.id;

        let Some(model) =
            UserRow::find_by_statement(self.db.get_database_backend().build(&params.into_select()))
                .one(&self.db)
                .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(model.into())
    }

    async fn create_user(
        &self,
        tx: &dyn Transaction,
        params: UserCreateParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        tx.execute(self.db.get_database_backend().build(&params.into_insert()))
            .await?;

        Ok(())
    }
}
