use colette_core::{users::UsersCreateData, User};
use colette_database::users::SelectByEmailParams;
use sqlx::PgExecutor;

#[derive(Clone, Debug)]
pub struct InsertParams<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

impl<'a> From<&'a UsersCreateData> for InsertParams<'a> {
    fn from(value: &'a UsersCreateData) -> Self {
        Self {
            email: &value.email,
            password: &value.password,
        }
    }
}

pub async fn select_by_email(
    ex: impl PgExecutor<'_>,
    params: SelectByEmailParams<'_>,
) -> Result<User, sqlx::Error> {
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

pub async fn insert(
    ex: impl PgExecutor<'_>,
    params: InsertParams<'_>,
) -> Result<User, sqlx::Error> {
    let row = sqlx::query_as!(
        User,
        "
   INSERT INTO users (email, password)
   VALUES ($1, $2)
RETURNING id,
          email,
          password,
          created_at,
          updated_at",
        params.email,
        params.password
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}