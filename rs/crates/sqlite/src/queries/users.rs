use colette_core::{users::UserCreateData, User};
use colette_database::users::SelectByEmailParams;
use sqlx::{Error, SqliteExecutor};
use uuid::Uuid;

#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: Uuid,
    pub email: &'a str,
    pub password: &'a str,
}

impl<'a> From<&'a UserCreateData> for InsertData<'a> {
    fn from(value: &'a UserCreateData) -> Self {
        Self {
            id: Uuid::new_v4(),
            email: &value.email,
            password: &value.password,
        }
    }
}

pub async fn select_by_email(
    ex: impl SqliteExecutor<'_>,
    params: SelectByEmailParams<'_>,
) -> Result<User, Error> {
    let row = sqlx::query_as!(
        User,
        "
SELECT id AS \"id: uuid::Uuid\",
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
RETURNING id AS \"id: uuid::Uuid\",
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
