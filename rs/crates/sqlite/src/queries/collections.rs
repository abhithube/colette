use colette_core::{collections::CollectionCreateData, Collection};
use colette_database::{
    collections::{SelectManyParams, UpdateData},
    FindOneParams,
};
use sqlx::SqliteExecutor;
use uuid::Uuid;

#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub is_default: bool,
    pub profile_id: &'a Uuid,
}

impl<'a> From<&'a CollectionCreateData> for InsertData<'a> {
    fn from(value: &'a CollectionCreateData) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: &value.title,
            is_default: false,
            profile_id: &value.profile_id,
        }
    }
}

pub async fn select_many(
    ex: impl SqliteExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Collection>, sqlx::Error> {
    let rows = sqlx::query_as!(
        Collection,
        "
SELECT c.id AS \"id: uuid::Uuid\",
       c.title,
       c.profile_id AS \"profile_id: uuid::Uuid\",
       c.created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
       c.updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\",
       count(b.collection_id) AS bookmark_count
  FROM collections AS c
       LEFT JOIN bookmarks AS b
       ON b.collection_id = c.id
 WHERE c.profile_id = $1
   AND NOT c.is_default
 GROUP BY c.id
 ORDER BY c.title ASC",
        params.profile_id
    )
    .fetch_all(ex)
    .await?;

    Ok(rows)
}

pub async fn select_by_id(
    ex: impl SqliteExecutor<'_>,
    params: FindOneParams<'_>,
) -> Result<Collection, sqlx::Error> {
    let row = sqlx::query_as!(
        Collection,
        "
SELECT c.id AS \"id: uuid::Uuid\",
       c.title,
       c.profile_id AS \"profile_id: uuid::Uuid\",
       c.created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
       c.updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\",
       count(b.collection_id) AS bookmark_count
  FROM collections AS c
       LEFT JOIN bookmarks AS b
       ON b.collection_id = c.id
 WHERE c.id = $1
   AND c.profile_id = $2
   AND NOT c.is_default
 GROUP BY c.id",
        params.id,
        params.profile_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn insert(
    ex: impl SqliteExecutor<'_>,
    data: InsertData<'_>,
) -> Result<Collection, sqlx::Error> {
    let row = sqlx::query_as!(
        Collection,
        "
   INSERT INTO collections (id, title, is_default, profile_id)
   VALUES ($1, $2, $3, $4)
RETURNING id AS \"id: uuid::Uuid\",
          title,
          profile_id AS \"profile_id: uuid::Uuid\",
          created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
          updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\",
          cast(0 AS bigint) AS bookmark_count",
        data.id,
        data.title,
        data.is_default,
        data.profile_id,
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn update(
    ex: impl SqliteExecutor<'_>,
    params: FindOneParams<'_>,
    data: UpdateData<'_>,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(
        "
UPDATE collections
   SET title = coalesce($3, title)
 WHERE id = $1
   AND profile_id = $2
   AND NOT is_default",
        params.id,
        params.profile_id,
        data.title,
    )
    .execute(ex)
    .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub async fn delete(
    ex: impl SqliteExecutor<'_>,
    params: FindOneParams<'_>,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(
        "
DELETE FROM collections
 WHERE id = $1
   AND profile_id = $2
   AND NOT is_default",
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
