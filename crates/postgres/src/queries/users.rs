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

impl<'a> From<&'a UserFindOneParams> for SelectByEmailParams<'a> {
    fn from(value: &'a UserFindOneParams) -> Self {
        Self {
            email: value.email.as_str(),
        }
    }
}

impl<'a> From<&'a UserCreateData> for InsertData<'a> {
    fn from(value: &'a UserCreateData) -> Self {
        Self {
            id: nanoid!(),
            email: value.email.as_str(),
            password: value.password.as_str(),
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
