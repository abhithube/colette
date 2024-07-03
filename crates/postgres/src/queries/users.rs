use colette_core::{
    users::{UserCreateData, UserFindOneParams},
    User,
};
use nanoid::nanoid;
use sqlx::{Error, PgExecutor};

#[derive(Debug)]
pub struct SelectByEmailParams<'a> {
    pub email: &'a str,
}

#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: String,
    pub email: &'a str,
    pub password: &'a str,
}

impl<'a> From<&UserFindOneParams<'a>> for SelectByEmailParams<'a> {
    fn from(value: &UserFindOneParams<'a>) -> Self {
        Self { email: value.email }
    }
}

impl<'a> From<&UserCreateData<'a>> for InsertData<'a> {
    fn from(value: &UserCreateData<'a>) -> Self {
        Self {
            id: nanoid!(),
            email: value.email,
            password: value.password,
        }
    }
}

pub async fn select_by_email(
    ex: impl PgExecutor<'_>,
    params: SelectByEmailParams<'_>,
) -> Result<User, Error> {
    let row = sqlx::query_file_as!(User, "queries/users/select_by_email.sql", params.email)
        .fetch_one(ex)
        .await?;

    Ok(row)
}

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<User, Error> {
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
