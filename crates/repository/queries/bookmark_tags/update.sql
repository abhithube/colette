WITH
  input_tags AS (
    SELECT
      *
    FROM
      unnest($2::UUID[]) AS t (id)
  ),
  deleted_bt AS (
    DELETE FROM bookmark_tags
    WHERE
      bookmark_id = $1
      AND NOT tag_id = ANY ($2)
  )
INSERT INTO
  bookmark_tags (bookmark_id, tag_id)
SELECT
  $1,
  id
FROM
  input_tags
ON CONFLICT (bookmark_id, tag_id) DO NOTHING
