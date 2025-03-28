use chrono::Utc;
use tokio_postgres::Row;
use torii_core::{NewUser, User, UserId, UserStorage, error::StorageError};

use super::PostgresBackend;

#[async_trait::async_trait]
impl UserStorage for PostgresBackend {
    type Error = StorageError;

    async fn create_user(&self, user: &NewUser) -> Result<User, Self::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let stmt = client
            .prepare_cached(
                "INSERT INTO users (id, email, name, verified_at, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
            )
            .await
            .map_err(|e| {
                StorageError::Database(e.to_string())
            })?;

        let row = client
            .query_one(
                &stmt,
                &[
                    &user.id.as_str(),
                    &user.email.as_str(),
                    &user.name.as_deref(),
                    &user.email_verified_at,
                    &Utc::now(),
                    &Utc::now(),
                ],
            )
            .await
            .map_err(|_| StorageError::Database("Failed to create user".to_string()))?;

        Ok(UserRow(&row).into())
    }

    async fn get_user(&self, id: &UserId) -> Result<Option<User>, Self::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let stmt = client
            .prepare_cached("SELECT * FROM users WHERE id = $1")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let row = client
            .query_opt(&stmt, &[&id.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to get user".to_string()))?;

        Ok(row.map(|e| UserRow(&e).into()))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, Self::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let stmt = client
            .prepare_cached("SELECT * FROM users WHERE email = $1")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let row = client
            .query_opt(&stmt, &[&email])
            .await
            .map_err(|_| StorageError::Database("Failed to get user by email".to_string()))?;

        Ok(row.map(|e| UserRow(&e).into()))
    }

    async fn get_or_create_user_by_email(&self, email: &str) -> Result<User, Self::Error> {
        let user = self.get_user_by_email(email).await?;
        if let Some(user) = user {
            return Ok(user);
        }

        let user = self
            .create_user(
                &NewUser::builder()
                    .id(UserId::new_random())
                    .email(email.into())
                    .build()
                    .unwrap(),
            )
            .await?;

        Ok(user)
    }

    async fn update_user(&self, user: &User) -> Result<User, Self::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let  stmt = client
            .prepare_cached("UPDATE users SET email = $1, name = $2, verified_at = $3, updated_at = $4 WHERE id = $5 RETURNING *")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let row = client
            .query_one(
                &stmt,
                &[
                    &user.email.as_str(),
                    &user.name.as_deref(),
                    &user.email_verified_at,
                    &user.updated_at,
                    &user.id.as_str(),
                ],
            )
            .await
            .map_err(|_| StorageError::Database("Failed to update user".into()))?;

        Ok(UserRow(&row).into())
    }

    async fn delete_user(&self, id: &UserId) -> Result<(), Self::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let stmt = client
            .prepare_cached("DELETE FROM users WHERE id = $1")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        client
            .execute(&stmt, &[&id.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to delete user".to_string()))?;

        Ok(())
    }

    async fn set_user_email_verified(&self, user_id: &UserId) -> Result<(), Self::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let stmt = client
            .prepare_cached("UPDATE users SET verified_at = $1 WHERE id = $2")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        client
            .execute(&stmt, &[&Utc::now(), &user_id.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to mark user as verified".into()))?;

        Ok(())
    }
}

struct UserRow<'a>(&'a Row);

impl From<UserRow<'_>> for User {
    fn from(UserRow(value): UserRow<'_>) -> Self {
        User::builder()
            .id(value.get::<_, String>("id").into())
            .email(value.get("email"))
            .email_verified_at(value.get("verified_at"))
            .name(value.get("name"))
            .created_at(value.get("created_at"))
            .updated_at(value.get("updated_at"))
            .build()
            .unwrap()
    }
}
