use torii_core::{UserId, error::StorageError, storage::PasswordStorage};

use super::LibsqlBackend;

#[async_trait::async_trait]
impl PasswordStorage for LibsqlBackend {
    type Error = StorageError;

    async fn set_password_hash(
        &self,
        user_id: &UserId,
        hash: &str,
    ) -> Result<(), <Self as PasswordStorage>::Error> {
        let mut stmt = self
            .conn
            .prepare("UPDATE users SET password_hash = ? WHERE id = ?")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        stmt.execute(libsql::params![hash, user_id.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to set password hash".to_string()))?;

        Ok(())
    }

    async fn get_password_hash(
        &self,
        user_id: &UserId,
    ) -> Result<Option<String>, <Self as PasswordStorage>::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT password_hash FROM users WHERE id = ?")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(libsql::params![user_id.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to get password hash".to_string()))?;
        let Some(row) = rows
            .next()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?
        else {
            return Ok(None);
        };

        Ok(Some(row.get::<String>(0).unwrap()))
    }
}
