use colette_core::{
    users::{Error, NotFoundError, UsersCreateData, UsersFindOneParams, UsersRepository},
    User,
};
use uuid::Uuid;

use crate::PostgresRepository;

#[async_trait::async_trait]
impl UsersRepository for PostgresRepository {
    async fn find_one_user(&self, params: UsersFindOneParams) -> Result<User, Error> {
        match params {
            UsersFindOneParams::Id(id) => {
                sqlx::query_file_as!(User, "queries/users/find_one.sql", id)
                    .fetch_one(self.db.get_postgres_connection_pool())
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound(NotFoundError::Id(id)),
                        _ => Error::Unknown(e.into()),
                    })
            }
            UsersFindOneParams::Email(email) => {
                sqlx::query_file_as!(User, "queries/users/find_by_email.sql", email)
                    .fetch_one(self.db.get_postgres_connection_pool())
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound(NotFoundError::Email(email)),
                        _ => Error::Unknown(e.into()),
                    })
            }
        }
    }

    async fn create_user(&self, data: UsersCreateData) -> Result<User, Error> {
        sqlx::query_file_as!(
            User,
            "queries/users/insert.sql",
            Uuid::new_v4(),
            data.email,
            data.password,
            Uuid::new_v4(),
        )
        .fetch_one(self.db.get_postgres_connection_pool())
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.email),
            _ => Error::Unknown(e.into()),
        })
    }
}
