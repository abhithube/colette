use colette_core::{
    User,
    common::Transaction,
    user::{Error, UserCreateParams, UserFindParams, UserRepository},
};
use colette_model::{UserRow, users};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, FromQueryResult,
    sea_query::{Asterisk, Expr, Query},
};

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
        let query = Query::select()
            .column(Asterisk)
            .from(users::Entity)
            .and_where(Expr::col(users::Column::Id).eq(params.id.to_string()))
            .to_owned();

        let Some(model) = UserRow::find_by_statement(self.db.get_database_backend().build(&query))
            .one(&self.db)
            .await?
        else {
            return Err(Error::NotFound(params.id));
        };

        Ok(model.into())
    }

    async fn create_user(
        &self,
        tx: &dyn Transaction,
        params: UserCreateParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::insert()
            .into_table(users::Entity)
            .columns([
                users::Column::Id,
                users::Column::Email,
                users::Column::DisplayName,
            ])
            .values_panic([
                params.id.to_string().into(),
                params.email.into(),
                params.display_name.into(),
            ])
            .to_owned();

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }
}
