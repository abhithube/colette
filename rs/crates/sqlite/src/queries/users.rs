use colette_core::User;
use colette_database::users::{InsertData, SelectByEmailParams};
use sqlx::{Error, SqliteExecutor};

pub async fn select_by_email(
    ex: impl SqliteExecutor<'_>,
    params: SelectByEmailParams<'_>,
) -> Result<User, Error> {
    let row = sqlx::query_as!(
        User,
        "
SELECT id,
       email,
       password,
       created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
       updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"
  FROM users
 WHERE email = $1",
        params.email
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn insert(ex: impl SqliteExecutor<'_>, data: InsertData<'_>) -> Result<User, Error> {
    let row = sqlx::query_as!(
        User,
        "
   INSERT INTO users (id, email, password)
   VALUES ($1, $2, $3)
RETURNING id,
          email,
          password,
          created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
          updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"",
        data.id,
        data.email,
        data.password
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}
