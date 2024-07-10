use colette_core::User;
use colette_database::users::{InsertData, SelectByEmailParams};
use sqlx::{Error, PgExecutor};

pub async fn select_by_email(
    ex: impl PgExecutor<'_>,
    params: SelectByEmailParams<'_>,
) -> Result<User, Error> {
    let row = sqlx::query_as!(
        User,
        "
SELECT id,
       email,
       password,
       created_at,
       updated_at
  FROM users
 WHERE email = $1",
        params.email
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<User, Error> {
    let row = sqlx::query_as!(
        User,
        "
   INSERT INTO users (id, email, password)
   VALUES ($1, $2, $3)
RETURNING id,
          email,
          password,
          created_at,
          updated_at",
        data.id,
        data.email,
        data.password
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}
