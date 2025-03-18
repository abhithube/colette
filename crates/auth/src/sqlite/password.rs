use colette_query::{
    IntoSelect, IntoUpdate,
    user::{UserSelectOne, UserUpdate},
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use torii_core::{UserId, error::StorageError, storage::PasswordStorage};

use super::{SqliteBackend, user::UserRow};

#[async_trait::async_trait]
impl PasswordStorage for SqliteBackend {
    type Error = StorageError;

    async fn set_password_hash(
        &self,
        user_id: &UserId,
        hash: &str,
    ) -> Result<(), <Self as PasswordStorage>::Error> {
        let (sql, values) = UserUpdate {
            id: user_id.as_str(),
            password_hash: Some(Some(hash)),
            ..Default::default()
        }
        .into_update()
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    async fn get_password_hash(
        &self,
        user_id: &UserId,
    ) -> Result<Option<String>, <Self as PasswordStorage>::Error> {
        let (sql, values) = UserSelectOne::Id(user_id.as_str())
            .into_select()
            .build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, UserRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(row.and_then(|e| e.password_hash))
    }
}
