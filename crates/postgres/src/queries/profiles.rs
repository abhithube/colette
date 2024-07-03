use colette_core::Profile;
use colette_database::profiles::{
    InsertData, SelectByIdParams, SelectDefaultParams, SelectManyParams, UpdateData,
};
use sqlx::{Error, PgExecutor};

pub async fn select_many(
    ex: impl PgExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Profile>, Error> {
    let rows = sqlx::query_file_as!(Profile, "queries/profiles/select_many.sql", params.user_id)
        .fetch_all(ex)
        .await?;

    Ok(rows)
}

pub async fn select_by_id(
    ex: impl PgExecutor<'_>,
    params: SelectByIdParams<'_>,
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
    params: SelectDefaultParams<'_>,
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

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<Profile, Error> {
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
    params: SelectByIdParams<'_>,
    data: UpdateData<'_>,
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
    params: SelectByIdParams<'_>,
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

pub async fn delete(ex: impl PgExecutor<'_>, params: SelectByIdParams<'_>) -> Result<(), Error> {
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
