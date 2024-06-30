use colette_core::Profile;
use sqlx::{Error, PgExecutor};

#[derive(Debug)]
pub struct SelectManyParams {
    pub user_id: String,
}

#[derive(Debug)]
pub struct SelectByIdParams {
    pub id: String,
    pub user_id: String,
}

#[derive(Debug)]
pub struct SelectDefaultParams {
    pub user_id: String,
}

#[derive(Debug)]
pub struct InsertData {
    pub id: String,
    pub title: String,
    pub image_url: Option<String>,
    pub is_default: bool,
    pub user_id: String,
}

#[derive(Debug)]
pub struct UpdateData {
    pub title: Option<String>,
    pub image_url: Option<String>,
}

pub async fn select_many(
    ex: impl PgExecutor<'_>,
    params: SelectManyParams,
) -> Result<Vec<Profile>, Error> {
    let rows = sqlx::query_file_as!(Profile, "queries/profiles/select_many.sql", params.user_id)
        .fetch_all(ex)
        .await?;

    Ok(rows)
}

pub async fn select_by_id(
    ex: impl PgExecutor<'_>,
    params: SelectByIdParams,
) -> Result<Profile, Error> {
    let row = sqlx::query_file_as!(
        Profile,
        "queries/profiles/select_by_id.sql",
        params.id,
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn select_default(
    ex: impl PgExecutor<'_>,
    params: SelectDefaultParams,
) -> Result<Profile, Error> {
    let row = sqlx::query_file_as!(
        Profile,
        "queries/profiles/select_default.sql",
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData) -> Result<Profile, Error> {
    let row = sqlx::query_file_as!(
        Profile,
        "queries/profiles/insert.sql",
        data.id,
        data.title,
        data.image_url,
        data.is_default,
        data.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn update(
    ex: impl PgExecutor<'_>,
    params: SelectByIdParams,
    data: UpdateData,
) -> Result<Profile, Error> {
    let row = sqlx::query_file_as!(
        Profile,
        "queries/profiles/update.sql",
        params.id,
        params.user_id,
        data.title,
        data.image_url,
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn update_default(
    ex: impl PgExecutor<'_>,
    params: SelectByIdParams,
) -> Result<Profile, Error> {
    let row = sqlx::query_file_as!(
        Profile,
        "queries/profiles/update_default.sql",
        params.id,
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn delete(ex: impl PgExecutor<'_>, params: SelectByIdParams) -> Result<(), Error> {
    sqlx::query_file_as!(
        Profile,
        "queries/profiles/delete.sql",
        params.id,
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(())
}
