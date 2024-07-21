use colette_core::{collections::CollectionCreateData, Collection};
use colette_database::{
    collections::{SelectManyParams, UpdateParams},
    SelectByIdParams,
};
use sqlx::{types::Uuid, PgExecutor};

#[derive(Clone, Debug)]
pub struct InsertParams<'a> {
    pub title: &'a str,
    pub is_default: bool,
    pub profile_id: &'a Uuid,
}

impl<'a> InsertParams<'a> {
    pub fn default_with_profile(profile_id: &'a Uuid) -> Self {
        Self {
            title: "Default",
            is_default: true,
            profile_id,
        }
    }
}

impl<'a> From<&'a CollectionCreateData> for InsertParams<'a> {
    fn from(value: &'a CollectionCreateData) -> Self {
        Self {
            title: &value.title,
            is_default: false,
            profile_id: &value.profile_id,
        }
    }
}

pub async fn select_many(
    ex: impl PgExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Collection>, sqlx::Error> {
    let rows = sqlx::query_as!(
        Collection,
        "
SELECT c.id,
       c.title,
       c.profile_id,
       c.created_at,
       c.updated_at,
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
    ex: impl PgExecutor<'_>,
    params: SelectByIdParams<'_>,
) -> Result<Collection, sqlx::Error> {
    let row = sqlx::query_as!(
        Collection,
        "
SELECT c.id,
       c.title,
       c.profile_id,
       c.created_at,
       c.updated_at,
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
    ex: impl PgExecutor<'_>,
    params: InsertParams<'_>,
) -> Result<Collection, sqlx::Error> {
    let row = sqlx::query_as!(
        Collection,
        "
   INSERT INTO collections (title, is_default, profile_id)
   VALUES ($1, $2, $3)
RETURNING id,
          title,
          profile_id,
          created_at,
          updated_at,
          cast(0 AS bigint) AS bookmark_count",
        params.title,
        params.is_default,
        params.profile_id,
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn update(
    ex: impl PgExecutor<'_>,
    params: UpdateParams<'_>,
) -> Result<Collection, sqlx::Error> {
    let row = sqlx::query_as!(
        Collection,
        "
  WITH
       c AS (
            UPDATE collections
               SET title = coalesce($3, title)
             WHERE id = $1
               AND profile_id = $2
               AND NOT is_default
         RETURNING id,
                   title,
                   profile_id,
                   created_at,
                   updated_at
       )
SELECT c.id,
       c.title,
       c.profile_id,
       c.created_at,
       c.updated_at,
       count(b.collection_id) AS bookmark_count
  FROM c
       LEFT JOIN bookmarks AS b
       ON b.collection_id = c.id
 GROUP BY c.id,
          c.title,
          c.profile_id,
          c.created_at,
          c.updated_at",
        params.id,
        params.profile_id,
        params.title,
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn delete(
    ex: impl PgExecutor<'_>,
    params: SelectByIdParams<'_>,
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
