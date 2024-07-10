use colette_core::Profile;
use colette_database::profiles::{
    InsertData, SelectByIdParams, SelectDefaultParams, SelectManyParams, UpdateData,
};
use futures::{Stream, StreamExt};
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

// pub async fn update_default(
//     ex: impl PgExecutor<'_>,
//     params: SelectByIdParams<'_>,
// ) -> Result<Profile, Error> {
//     let row = sqlx::query_file_as!(
//         Profile,
//         "queries/profiles/update_default.sql",
//         params.id,
//         params.user_id
//     )
//     .fetch_one(ex)
//     .await?;

//     Ok(row)
// }

pub async fn delete(ex: impl PgExecutor<'_>, params: SelectByIdParams<'_>) -> Result<(), Error> {
    sqlx::query!(
        "DELETE FROM profiles WHERE id = $1 AND user_id = $2",
        params.id,
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(())
}

pub fn iterate<'a>(
    ex: impl PgExecutor<'a> + 'a,
    feed_id: i64,
) -> impl Stream<Item = Result<String, Error>> + 'a {
    sqlx::query!(
        "
SELECT p.id
FROM profiles p
JOIN profile_feeds pf ON pf.profile_id = p.id
WHERE pf.feed_id = $1",
        feed_id
    )
    .fetch(ex)
    .map(|e| e.map(|e| e.id))
}
