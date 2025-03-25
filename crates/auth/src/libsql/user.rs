use chrono::{DateTime, Utc};
use torii_core::{NewUser, User, UserId, UserStorage, error::StorageError};

use super::LibsqlBackend;

#[async_trait::async_trait]
impl UserStorage for LibsqlBackend {
    type Error = StorageError;

    async fn create_user(&self, user: &NewUser) -> Result<User, Self::Error> {
        let mut stmt = self
            .conn
            .prepare(
                "INSERT INTO users (id, email, name, verified_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?) RETURNING *",
            )
            .await
            .map_err(|e| {
                StorageError::Database(e.to_string())
            })?;

        let row = stmt
            .query_row(libsql::params![
                user.id.as_str(),
                user.email.as_str(),
                user.name.as_deref(),
                user.email_verified_at.map(|e| e.timestamp()),
                Utc::now().timestamp(),
                Utc::now().timestamp(),
            ])
            .await
            .map_err(|_| StorageError::Database("Failed to create user".to_string()))?;

        let row = libsql::de::from_row::<UserRow>(&row)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(row.into())
    }

    async fn get_user(&self, id: &UserId) -> Result<Option<User>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM users WHERE id = ?")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(libsql::params![id.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to get user".to_string()))?;
        let Some(row) = rows
            .next()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?
        else {
            return Ok(None);
        };

        let row = libsql::de::from_row::<UserRow>(&row)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(Some(row.into()))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM users WHERE email = ?")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(libsql::params![email])
            .await
            .map_err(|_| StorageError::Database("Failed to get user by email".to_string()))?;
        let Some(row) = rows
            .next()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?
        else {
            return Ok(None);
        };

        let row = libsql::de::from_row::<UserRow>(&row)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(Some(row.into()))
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
        let mut stmt = self
            .conn
            .prepare("UPDATE users SET email = ?, name = ?, verified_at = ?, updated_at = ? WHERE id = ? RETURNING *")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let row = stmt
            .query_row(libsql::params![
                user.email.as_str(),
                user.name.as_deref(),
                user.email_verified_at.map(|e| e.timestamp()),
                user.updated_at.timestamp(),
                user.id.as_str()
            ])
            .await
            .map_err(|_| StorageError::Database("Failed to update user".into()))?;

        let row = libsql::de::from_row::<UserRow>(&row)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(row.into())
    }

    async fn delete_user(&self, id: &UserId) -> Result<(), Self::Error> {
        let mut stmt = self
            .conn
            .prepare("DELETE FROM users WHERE id = ?")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        stmt.execute(libsql::params![id.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to delete user".to_string()))?;

        Ok(())
    }

    async fn set_user_email_verified(&self, user_id: &UserId) -> Result<(), Self::Error> {
        let mut stmt = self
            .conn
            .prepare("UPDATE users SET verified_at = ? WHERE id = ?")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        stmt.execute(libsql::params![Utc::now().timestamp(), user_id.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to mark user as verified".into()))?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct UserRow {
    id: String,
    name: Option<String>,
    email: String,
    verified_at: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

impl From<UserRow> for User {
    fn from(value: UserRow) -> Self {
        User::builder()
            .id(value.id.into())
            .email(value.email)
            .email_verified_at(
                value
                    .verified_at
                    .and_then(|e| DateTime::from_timestamp(e, 0)),
            )
            .name(value.name)
            .created_at(DateTime::from_timestamp(value.created_at, 0).unwrap())
            .updated_at(DateTime::from_timestamp(value.updated_at, 0).unwrap())
            .build()
            .unwrap()
    }
}
