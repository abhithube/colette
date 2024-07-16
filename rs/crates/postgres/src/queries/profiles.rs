use colette_core::{common::SendableStream, profiles::ProfileCreateData, Profile};
use colette_database::profiles::{
    SelectByIdParams, SelectDefaultParams, SelectManyParams, UpdateData,
};
use sqlx::{postgres::PgRow, types::Uuid, Error, PgExecutor, Row};

#[derive(Debug)]
pub struct InsertData<'a> {
    pub title: &'a str,
    pub image_url: Option<&'a str>,
    pub is_default: bool,
    pub user_id: &'a Uuid,
}

impl<'a> InsertData<'a> {
    pub fn default_with_user(user_id: &'a Uuid) -> Self {
        Self {
            title: "Default",
            image_url: None,
            is_default: true,
            user_id,
        }
    }
}

impl<'a> From<&'a ProfileCreateData> for InsertData<'a> {
    fn from(value: &'a ProfileCreateData) -> Self {
        Self {
            title: &value.title,
            image_url: value.image_url.as_deref(),
            is_default: false,
            user_id: &value.user_id,
        }
    }
}

pub async fn select_many(
    ex: impl PgExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Profile>, Error> {
    let rows = sqlx::query_as!(
        Profile,
        "
SELECT id,
       title,
       image_url,
       user_id,
       created_at,
       updated_at
  FROM profiles
 WHERE user_id = $1",
        params.user_id
    )
    .fetch_all(ex)
    .await?;

    Ok(rows)
}

pub async fn select_by_id(
    ex: impl PgExecutor<'_>,
    params: SelectByIdParams<'_>,
) -> Result<Profile, Error> {
    let row = sqlx::query_as!(
        Profile,
        "
SELECT id,
       title,
       image_url,
       user_id,
       created_at,
       updated_at
  FROM profiles
 WHERE id = $1
   AND user_id = $2",
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
    let row = sqlx::query_as!(
        Profile,
        "
SELECT id,
       title,
       image_url,
       user_id,
       created_at,
       updated_at
  FROM profiles
 WHERE user_id = $1
   AND is_default = TRUE",
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<Profile, Error> {
    let row = sqlx::query_as!(
        Profile,
        "
   INSERT INTO profiles (title, image_url, is_default, user_id)
   VALUES ($1, $2, $3, $4)
RETURNING id,
          title,
          image_url,
          user_id,
          created_at,
          updated_at",
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
    let row = sqlx::query_as!(
        Profile,
        "
   UPDATE profiles
      SET title = coalesce($3, title),
          image_url = coalesce($4, image_url)
    WHERE id = $1
      AND user_id = $2
RETURNING id,
          title,
          image_url,
          user_id,
          created_at,
          updated_at",
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
//     let row = sqlx::query_as!(
//         Profile,
//         "
//      WITH
//           u AS (
//             UPDATE profiles
//                SET is_default = FALSE
//              WHERE user_id = $2
//                AND is_default = TRUE
//           )
//    UPDATE profiles
//       SET is_default = TRUE
//     WHERE id = $1
//       AND user_id = $2
// RETURNING id,
//           title,
//           image_url,
//           user_id,
//           created_at,
//           updated_at",
//         params.id,
//         params.user_id
//     )
//     .fetch_one(ex)
//     .await?;

//     Ok(row)
// }

pub async fn delete(ex: impl PgExecutor<'_>, params: SelectByIdParams<'_>) -> Result<(), Error> {
    let result = sqlx::query!(
        "
DELETE FROM profiles
 WHERE id = $1
   AND user_id = $2",
        params.id,
        params.user_id
    )
    .execute(ex)
    .await?;

    if result.rows_affected() == 0 {
        return Err(Error::RowNotFound);
    }

    Ok(())
}

pub fn iterate<'a>(
    ex: impl PgExecutor<'a> + 'a,
    feed_id: i64,
) -> SendableStream<'a, Result<Uuid, Error>> {
    sqlx::query(
        "
SELECT p.id
  FROM profiles AS p
  JOIN profile_feeds AS pf
    ON pf.profile_id = p.id
 WHERE pf.feed_id = $1",
    )
    .bind(feed_id)
    .map(|e: PgRow| e.get(0))
    .fetch(ex)
}
