WITH
  upserted_bookmark AS (
    INSERT INTO
      bookmarks (
        id,
        link,
        title,
        thumbnail_url,
        published_at,
        author,
        user_id,
        created_at,
        updated_at
      )
    VALUES
      ($1, $2, $3, $4, $5, $6, $8, $9, $10)
    ON CONFLICT (id) DO UPDATE
    SET
      title = EXCLUDED.title,
      thumbnail_url = EXCLUDED.thumbnail_url,
      published_at = EXCLUDED.published_at,
      author = EXCLUDED.author,
      updated_at = EXCLUDED.updated_at
  ),
  input_tags AS (
    SELECT
      *
    FROM
      unnest($7::UUID[]) AS t (id)
  ),
  deleted_bt AS (
    DELETE FROM bookmark_tags
    WHERE
      bookmark_id = $1
      AND NOT tag_id = ANY ($7)
  )
INSERT INTO
  bookmark_tags (bookmark_id, tag_id, created_at, updated_at)
SELECT
  $1,
  id,
  now(),
  now()
FROM
  input_tags
ON CONFLICT (bookmark_id, tag_id) DO NOTHING
