use colette_core::Profile;
use colette_database::profiles::{
    InsertData, SelectByIdParams, SelectDefaultParams, SelectManyParams, UpdateData,
};
use sqlx::{Error, SqliteExecutor};

// #[derive(Debug)]
// pub struct UpdateDefaultUnsetParams<'a> {
//     pub user_id: &'a str,
// }

pub async fn select_many(
    ex: impl SqliteExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Profile>, Error> {
    let rows = sqlx::query_file_as!(Profile, "queries/profiles/select_many.sql", params.user_id)
        .fetch_all(ex)
        .await?;

    Ok(rows)
}

pub async fn select_by_id(
    ex: impl SqliteExecutor<'_>,
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
    ex: impl SqliteExecutor<'_>,
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

pub async fn insert(ex: impl SqliteExecutor<'_>, data: InsertData<'_>) -> Result<Profile, Error> {
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

// pub async fn update_default_set(
//     ex: impl SqliteExecutor<'_>,
//     params: SelectByIdParams<'_>,
// ) -> Result<Profile, Error> {
//     let row = sqlx::query_file_as!(
//         Profile,
//         "queries/profiles/update_default_set.sql",
//         params.id,
//         params.user_id
//     )
//     .fetch_one(ex)
//     .await?;

//     Ok(row)
// }

// pub async fn update_default_unset(
//     ex: impl SqliteExecutor<'_>,
//     params: UpdateDefaultUnsetParams<'_>,
// ) -> Result<String, Error> {
//     let row = sqlx::query_file!("queries/profiles/update_default_unset.sql", params.user_id)
//         .fetch_one(ex)
//         .await?;

//     Ok(row.id)
// }

pub async fn delete(
    ex: impl SqliteExecutor<'_>,
    params: SelectByIdParams<'_>,
) -> Result<(), Error> {
    sqlx::query!(
        "DELETE FROM profiles WHERE id = $1 AND user_id = $2",
        params.id,
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(())
}
