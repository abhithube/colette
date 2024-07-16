use colette_core::{users::UserCreateData, User};
use colette_database::users::SelectByEmailParams;
use sqlx::PgExecutor;

#[derive(Debug)]
pub struct InsertData<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

impl<'a> From<&'a UserCreateData> for InsertData<'a> {
    fn from(value: &'a UserCreateData) -> Self {
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

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<User, sqlx::Error> {
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
        data.email,
        data.password
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}
