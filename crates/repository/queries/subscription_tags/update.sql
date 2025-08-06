WITH
  input_tags AS (
    SELECT
      *
    FROM
      unnest($2::UUID[]) AS t (id)
  ),
  deleted_st AS (
    DELETE FROM subscription_tags
    WHERE
      subscription_id = $1
      AND NOT tag_id = ANY ($2)
  )
INSERT INTO
  subscription_tags (subscription_id, tag_id)
SELECT
  $1,
  id
FROM
  input_tags
ON CONFLICT (subscription_id, tag_id) DO NOTHING
