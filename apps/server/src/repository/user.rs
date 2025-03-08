use colette_core::{
    User,
    user::{Error, UserFindParams, UserRepository},
};
use colette_model::{UserRow, users};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, FromQueryResult,
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
}
