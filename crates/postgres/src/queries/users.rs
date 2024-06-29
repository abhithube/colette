use colette_core::users::{CreateData, User};
use nanoid::nanoid;
use sqlx::{Error, PgExecutor};

#[derive(Debug)]
pub struct InsertData {
    pub id: String,
    pub email: String,
    pub password: String,
}

impl From<CreateData> for InsertData {
    fn from(value: CreateData) -> Self {
        Self {
            id: nanoid!(),
            email: value.email,
            password: value.password,
        }
    }
}

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData) -> Result<User, Error> {
    let row = sqlx::query_file_as!(
        User,
        "queries/users/insert.sql",
        data.id,
        data.email,
        data.password
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}
