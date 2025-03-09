use colette_core::{
    User,
    common::Transaction,
    user::{Error, UserCreateParams, UserFindParams, UserRepository},
};
use colette_query::{IntoInsert, IntoSelect};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, FromQueryResult};

use super::common::parse_timestamp;

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

#[derive(sea_orm::FromQueryResult)]
struct UserRow {
    id: String,
    email: String,
    display_name: Option<String>,
    created_at: i32,
    updated_at: i32,
}

impl From<UserRow> for User {
    fn from(value: UserRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            email: value.email,
            display_name: value.display_name,
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
        }
    }
}
