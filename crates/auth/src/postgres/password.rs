use torii_core::{UserId, error::StorageError, storage::PasswordStorage};

use super::PostgresBackend;

#[async_trait::async_trait]
impl PasswordStorage for PostgresBackend {
    type Error = StorageError;

    async fn set_password_hash(
        &self,
        user_id: &UserId,
        hash: &str,
    ) -> Result<(), <Self as PasswordStorage>::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let stmt = client
            .prepare_cached("UPDATE users SET password_hash = $1 WHERE id = $2")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        client
            .execute(&stmt, &[&hash, &user_id.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to set password hash".to_string()))?;

        Ok(())
    }

    async fn get_password_hash(
        &self,
        user_id: &UserId,
    ) -> Result<Option<String>, <Self as PasswordStorage>::Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let stmt = client
            .prepare_cached("SELECT password_hash FROM users WHERE id = $1")
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let row = client
            .query_opt(&stmt, &[&user_id.as_str()])
            .await
            .map_err(|_| StorageError::Database("Failed to get password hash".to_string()))?;

        Ok(row.map(|e| e.get("password_hash")))
    }
}
