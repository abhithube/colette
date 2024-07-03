use colette_core::User;
use colette_database::users::{InsertData, SelectByEmailParams};
use sqlx::{Error, PgExecutor};

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
