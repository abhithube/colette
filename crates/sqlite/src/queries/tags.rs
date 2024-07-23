use colette_core::{tags::TagsCreateData, Tag};
use colette_database::{tags::UpdateParams, SelectByIdParams, SelectManyParams};
use sqlx::{
    types::{chrono, Uuid},
    SqliteExecutor,
};

#[derive(Clone, Debug)]
pub struct InsertParams<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub profile_id: &'a Uuid,
}

impl<'a> From<&'a TagsCreateData> for InsertParams<'a> {
    fn from(value: &'a TagsCreateData) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: &value.title,
            profile_id: &value.profile_id,
        }
    }
}

pub async fn select_many(
    ex: impl SqliteExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Tag>, sqlx::Error> {
    let rows = sqlx::query_as!(
        Tag,
        "
SELECT id AS \"id: uuid::Uuid\",
       title,
       profile_id AS \"profile_id: uuid::Uuid\",
       created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
       updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"
  FROM tags
 WHERE profile_id = $1
 ORDER BY title ASC",
        params.profile_id
    )
    .fetch_all(ex)
    .await?;

    Ok(rows)
}

pub async fn select_by_id(
    ex: impl SqliteExecutor<'_>,
    params: SelectByIdParams<'_>,
) -> Result<Tag, sqlx::Error> {
    let row = sqlx::query_as!(
        Tag,
        "
SELECT id AS \"id: uuid::Uuid\",
       title,
       profile_id AS \"profile_id: uuid::Uuid\",
       created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
       updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"
  FROM tags
 WHERE id = $1
   AND profile_id = $2",
        params.id,
        params.profile_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn insert(
    ex: impl SqliteExecutor<'_>,
    params: InsertParams<'_>,
) -> Result<Tag, sqlx::Error> {
    let row = sqlx::query_as!(
        Tag,
        "
   INSERT INTO tags (id, title, profile_id)
   VALUES ($1, $2, $3)
RETURNING id AS \"id: uuid::Uuid\",
          title,
          profile_id AS \"profile_id: uuid::Uuid\",
          created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
          updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"",
        params.id,
        params.title,
        params.profile_id,
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn update(
    ex: impl SqliteExecutor<'_>,
    params: UpdateParams<'_>,
) -> Result<Tag, sqlx::Error> {
    let row = sqlx::query_as!(
        Tag,
        "
   UPDATE tags
      SET title = coalesce($3, title)
    WHERE id = $1
      AND profile_id = $2
RETURNING id AS \"id: uuid::Uuid\",
          title,
          profile_id AS \"profile_id: uuid::Uuid\",
          created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
          updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"",
        params.id,
        params.profile_id,
        params.title,
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn delete(
    ex: impl SqliteExecutor<'_>,
    params: SelectByIdParams<'_>,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(
        "
DELETE FROM tags
 WHERE id = $1
   AND profile_id = $2",
        params.id,
        params.profile_id
    )
    .execute(ex)
    .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}
