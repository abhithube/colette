use chrono::{DateTime, Utc};
use colette_core::{
    User,
    user::{Error, UserInsertParams, UserRepository, UserUpdateParams},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::DbUrl;

#[derive(Debug, Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Error> {
        let user = sqlx::query_file_as!(UserRow, "queries/users/find_by_id.sql", id)
            .map(Into::into)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn find_by_email(&self, email: String) -> Result<Option<User>, Error> {
        let user = sqlx::query_file_as!(UserRow, "queries/users/find_by_email.sql", email)
            .map(Into::into)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    async fn insert(&self, params: UserInsertParams) -> Result<Uuid, Error> {
        let mut tx = self.pool.begin().await?;

        let id = sqlx::query_file_scalar!(
            "queries/users/insert.sql",
            params.email,
            params.display_name,
            params.image_url.map(Into::into) as Option<DbUrl>
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query_file_scalar!(
            "queries/accounts/insert.sql",
            params.sub,
            params.provider,
            params.password_hash,
            id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(id)
    }

    async fn update(&self, params: UserUpdateParams) -> Result<(), Error> {
        let (has_display_name, display_name) = if let Some(display_name) = params.display_name {
            (true, display_name)
        } else {
            (false, None)
        };
        let (has_image_url, image_url) = if let Some(image_url) = params.image_url {
            (true, image_url)
        } else {
            (false, None)
        };

        sqlx::query_file!(
            "queries/users/update.sql",
            params.id,
            has_display_name,
            display_name,
            has_image_url,
            image_url.map(Into::into) as Option<DbUrl>
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

struct UserRow {
    id: Uuid,
    email: String,
    display_name: Option<String>,
    image_url: Option<DbUrl>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(value: UserRow) -> Self {
        Self {
            id: value.id,
            email: value.email,
            display_name: value.display_name,
            image_url: value.image_url.map(Into::into),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
