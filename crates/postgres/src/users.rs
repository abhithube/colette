use colette_core::{
    users::{Error, UsersCreateData, UsersFindOneParams, UsersRepository},
    User,
};

use crate::PostgresRepository;

#[async_trait::async_trait]
impl UsersRepository for PostgresRepository {
    async fn find_one_user(&self, params: UsersFindOneParams) -> Result<User, Error> {
        sqlx::query_file_as!(User, "queries/users/find_one.sql", params.email)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn create_user(&self, data: UsersCreateData) -> Result<User, Error> {
        sqlx::query_file_as!(User, "queries/users/insert.sql", data.email, data.password)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }
}
