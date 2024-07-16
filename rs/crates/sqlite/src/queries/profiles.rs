use colette_core::{profiles::ProfileCreateData, Profile};
use colette_database::profiles::{
    SelectByIdParams, SelectDefaultParams, SelectManyParams, UpdateData,
};
use futures::Stream;
use sqlx::{sqlite::SqliteRow, Error, Row, SqliteExecutor};
use uuid::Uuid;

#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub image_url: Option<&'a str>,
    pub is_default: bool,
    pub user_id: &'a Uuid,
}

impl<'a> InsertData<'a> {
    pub fn default_with_user(user_id: &'a Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
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
            id: Uuid::new_v4(),
            title: &value.title,
            image_url: value.image_url.as_deref(),
            is_default: false,
            user_id: &value.user_id,
        }
    }
}

// #[derive(Debug)]
// pub struct UpdateDefaultUnsetParams<'a> {
//     pub user_id: &'a str,
// }

pub async fn select_many(
    ex: impl SqliteExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Profile>, Error> {
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
    params: SelectByIdParams<'_>,
) -> Result<Profile, Error> {
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
    params: SelectDefaultParams<'_>,
) -> Result<Profile, Error> {
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

pub async fn insert(ex: impl SqliteExecutor<'_>, data: InsertData<'_>) -> Result<Profile, Error> {
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
// ) -> Result<Uuid, Error> {
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
    params: SelectByIdParams<'_>,
) -> Result<(), Error> {
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
    ex: impl SqliteExecutor<'a> + 'a,
    feed_id: i64,
) -> impl Stream<Item = Result<Uuid, Error>> + 'a {
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
