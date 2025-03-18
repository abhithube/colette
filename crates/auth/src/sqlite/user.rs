use chrono::{DateTime, Utc};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect, IntoUpdate,
    user::{UserDelete, UserInsert, UserSelectOne, UserUpdate},
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use torii_core::{NewUser, User, UserId, UserStorage, error::StorageError};

use super::SqliteBackend;

#[async_trait::async_trait]
impl UserStorage for SqliteBackend {
    type Error = StorageError;

    async fn create_user(&self, user: &NewUser) -> Result<User, Self::Error> {
        let (sql, values) = UserInsert {
            id: user.id.as_str(),
            email: &user.email,
            verified_at: user.email_verified_at,
            name: user.name.as_deref(),
            ..Default::default()
        }
        .into_insert()
        .build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, UserRow, _>(&sql, values)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| StorageError::Database("Failed to create user".into()))?;

        Ok(row.into())
    }

    async fn get_user(&self, id: &UserId) -> Result<Option<User>, Self::Error> {
        let (sql, values) = UserSelectOne::Id(id.as_str())
            .into_select()
            .build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, UserRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| StorageError::Database("Failed to get user".into()))?;

        Ok(row.map(Into::into))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, Self::Error> {
        let (sql, values) = UserSelectOne::Email(email)
            .into_select()
            .build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, UserRow, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| StorageError::Database("Failed to get user by email".into()))?;

        Ok(row.map(Into::into))
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
            .await
            .map_err(|_| StorageError::Database("Failed to get or create user by email".into()))?;

        Ok(user)
    }

    async fn update_user(&self, user: &User) -> Result<User, Self::Error> {
        let (sql, values) = UserUpdate {
            id: user.id.as_str(),
            name: Some(user.name.as_deref()),
            email: Some(&user.email),
            ..Default::default()
        }
        .into_update()
        .build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, UserRow, _>(&sql, values)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| StorageError::Database("Failed to update user".into()))?;

        Ok(row.into())
    }

    async fn delete_user(&self, id: &UserId) -> Result<(), Self::Error> {
        let (sql, values) = UserDelete { id: id.as_str() }
            .into_delete()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|_| StorageError::Database("Failed to delete user".into()))?;

        Ok(())
    }

    async fn set_user_email_verified(&self, user_id: &UserId) -> Result<(), Self::Error> {
        let (sql, values) = UserUpdate {
            id: user_id.as_str(),
            verified_at: Some(Some(Utc::now())),
            ..Default::default()
        }
        .into_update()
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|_| StorageError::Database("Failed to set user email verified".into()))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
pub(crate) struct UserRow {
    id: String,
    name: Option<String>,
    email: String,
    verified_at: Option<DateTime<Utc>>,
    pub(crate) password_hash: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(value: UserRow) -> Self {
        Self {
            id: UserId::new(&value.id),
            name: value.name,
            email: value.email,
            email_verified_at: value.verified_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<User> for UserRow {
    fn from(value: User) -> Self {
        Self {
            id: value.id.into_inner(),
            name: value.name,
            email: value.email,
            verified_at: value.email_verified_at,
            password_hash: None,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
