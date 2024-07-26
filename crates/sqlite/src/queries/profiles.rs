use colette_core::{common::SendableStream, profiles::ProfilesCreateData, Profile};
use colette_database::profiles::{
    SelectByIdParams, SelectDefaultParams, SelectManyParams, UpdateParams,
};
use sqlx::{sqlite::SqliteRow, types::chrono, Row, SqliteExecutor};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct InsertParams<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub image_url: Option<&'a str>,
    pub is_default: bool,
    pub user_id: Uuid,
}

impl<'a> InsertParams<'a> {
    pub fn default_with_user(user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: "Default",
            image_url: None,
            is_default: true,
            user_id,
        }
    }
}

impl<'a> From<&'a ProfilesCreateData> for InsertParams<'a> {
    fn from(value: &'a ProfilesCreateData) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: &value.title,
            image_url: value.image_url.as_deref(),
            is_default: false,
            user_id: value.user_id,
        }
    }
}

// #[derive(Clone, Debug)]
// pub struct UpdateDefaultUnsetParams<'a> {
//     pub user_id: &'a str,
// }

pub async fn select_many(
    ex: impl SqliteExecutor<'_>,
    params: SelectManyParams,
) -> Result<Vec<Profile>, sqlx::Error> {
    let rows = sqlx::query_as!(
        Profile,
        "
SELECT id AS \"id: uuid::Uuid\",
       title,
       image_url,
       user_id AS \"user_id: uuid::Uuid\",
       created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
       updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"
  FROM profiles
 WHERE user_id = $1",
        params.user_id
    )
    .fetch_all(ex)
    .await?;

    Ok(rows)
}

pub async fn select_by_id(
    ex: impl SqliteExecutor<'_>,
    params: SelectByIdParams,
) -> Result<Profile, sqlx::Error> {
    let row = sqlx::query_as!(
        Profile,
        "
SELECT id AS \"id: uuid::Uuid\",
       title,
       image_url,
       user_id AS \"user_id: uuid::Uuid\",
       created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
       updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"
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
    ex: impl SqliteExecutor<'_>,
    params: SelectDefaultParams,
) -> Result<Profile, sqlx::Error> {
    let row = sqlx::query_as!(
        Profile,
        "
SELECT id AS \"id: uuid::Uuid\",
       title,
       image_url,
       user_id AS \"user_id: uuid::Uuid\",
       created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
       updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"
  FROM profiles
 WHERE user_id = $1
   AND is_default = 1",
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn insert(
    ex: impl SqliteExecutor<'_>,
    params: InsertParams<'_>,
) -> Result<Profile, sqlx::Error> {
    let row = sqlx::query_as!(
        Profile,
        "
   INSERT INTO profiles (id, title, image_url, is_default, user_id)
   VALUES ($1, $2, $3, $4, $5)
RETURNING id AS \"id: uuid::Uuid\",
          title,
          image_url,
          user_id AS \"user_id: uuid::Uuid\",
          created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
          updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"",
        params.id,
        params.title,
        params.image_url,
        params.is_default,
        params.user_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn update(
    ex: impl SqliteExecutor<'_>,
    params: UpdateParams<'_>,
) -> Result<Profile, sqlx::Error> {
    let row = sqlx::query_as!(
        Profile,
        "
   UPDATE profiles
      SET title = coalesce($3, title),
          image_url = coalesce($4, image_url)
    WHERE id = $1
      AND user_id = $2
RETURNING id AS \"id: uuid::Uuid\",
          title,
          image_url,
          user_id AS \"user_id: uuid::Uuid\",
          created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
          updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"",
        params.id,
        params.user_id,
        params.title,
        params.image_url,
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

// pub async fn update_default_set(
//     ex: impl SqliteExecutor<'_>,
//     params: SelectByIdParams<'_>,
// ) -> Result<Profile, sqlx::Error> {
//     let row = sqlx::query_as!(
//         Profile,
//         "
//    UPDATE profiles
//       SET is_default = 1
//     WHERE id = $1
//       AND user_id = $2
// RETURNING id AS \"id: uuid::Uuid\",
//           title,
//           image_url,
//           user_id AS \"user_id: uuid::Uuid\",
//           created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
//           updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"",
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
// ) -> Result<Uuid, sqlx::Error> {
//     let row = sqlx::query!(
//         "
//    UPDATE profiles
//       SET is_default = 0
//     WHERE user_id = $1
//       AND is_default = 1
// RETURNING id AS \"id: uuid::Uuid\"",
//         params.user_id
//     )
//     .fetch_one(ex)
//     .await?;

//     Ok(row.id)
// }

pub async fn delete(
    ex: impl SqliteExecutor<'_>,
    params: SelectByIdParams,
) -> Result<(), sqlx::Error> {
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
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub fn iterate<'a>(
    ex: impl SqliteExecutor<'a> + 'a,
    feed_id: i64,
) -> SendableStream<'a, Result<Uuid, sqlx::Error>> {
    sqlx::query(
        "
SELECT p.id AS \"id: uuid::Uuid\"
  FROM profiles AS p
  JOIN profile_feeds AS pf
    ON pf.profile_id = p.id
 WHERE pf.feed_id = $1",
    )
    .bind(feed_id)
    .map(|row: SqliteRow| row.get(0))
    .fetch(ex)
}
