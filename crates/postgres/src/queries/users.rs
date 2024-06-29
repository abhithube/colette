use colette_core::users::{CreateData, FindOneParams, User};
use nanoid::nanoid;
use sqlx::{Error, PgExecutor};

#[derive(Debug)]
pub struct SelectByEmailParams {
    pub email: String,
}

#[derive(Debug)]
pub struct InsertData {
    pub id: String,
    pub email: String,
    pub password: String,
}

impl From<FindOneParams> for SelectByEmailParams {
    fn from(value: FindOneParams) -> Self {
        Self { email: value.email }
    }
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

pub async fn select_by_email(
    ex: impl PgExecutor<'_>,
    params: SelectByEmailParams,
) -> Result<User, Error> {
    let row = sqlx::query_file_as!(User, "queries/users/select_by_email.sql", params.email)
        .fetch_one(ex)
        .await?;

    Ok(row)
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
