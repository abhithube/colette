use colette_core::Profile;
use sqlx::{Error, SqliteExecutor};

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
pub struct UpdateDefaultUnsetParams {
    pub user_id: String,
}

#[derive(Debug)]
pub struct UpdateData {
    pub title: Option<String>,
    pub image_url: Option<String>,
}

pub async fn select_many(
    ex: impl SqliteExecutor<'_>,
    params: SelectManyParams,
) -> Result<Vec<Profile>, Error> {
    let rows = sqlx::query_file_as!(Profile, "queries/profiles/select_many.sql", params.user_id)
        .fetch_all(ex)
        .await?;

    Ok(rows)
}

pub async fn select_by_id(
    ex: impl SqliteExecutor<'_>,
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
    ex: impl SqliteExecutor<'_>,
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

pub async fn insert(ex: impl SqliteExecutor<'_>, data: InsertData) -> Result<Profile, Error> {
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
    ex: impl SqliteExecutor<'_>,
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

pub async fn update_default_set(
    ex: impl SqliteExecutor<'_>,
    params: SelectByIdParams,
) -> Result<Profile, Error> {
    let row = sqlx::query_file_as!(
        Profile,
        "queries/profiles/update_default_set.sql",
        params.id,
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn update_default_unset(
    ex: impl SqliteExecutor<'_>,
    params: UpdateDefaultUnsetParams,
) -> Result<String, Error> {
    let row = sqlx::query_file!("queries/profiles/update_default_unset.sql", params.user_id)
        .fetch_one(ex)
        .await?;

    Ok(row.id)
}

pub async fn delete(ex: impl SqliteExecutor<'_>, params: SelectByIdParams) -> Result<(), Error> {
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
