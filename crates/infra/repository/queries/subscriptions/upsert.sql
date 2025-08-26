WITH
  upserted_subscription AS (
    INSERT INTO
      subscriptions (
        id,
        title,
        description,
        feed_id,
        user_id,
        created_at,
        updated_at
      )
    VALUES
      ($1, $2, $3, $4, $6, $7, $8)
    ON CONFLICT (id) DO UPDATE
    SET
      title = EXCLUDED.title,
      description = EXCLUDED.description,
      updated_at = EXCLUDED.updated_at
  ),
  input_tags AS (
    SELECT
      *
    FROM
      unnest($5::UUID[]) AS t (id)
  ),
  deleted_st AS (
    DELETE FROM subscription_tags
    WHERE
      subscription_id = $1
      AND NOT tag_id = ANY ($5)
  )
INSERT INTO
  subscription_tags (subscription_id, tag_id, created_at, updated_at)
SELECT
  $1,
  id,
  now(),
  now()
FROM
  input_tags
ON CONFLICT (subscription_id, tag_id) DO NOTHING
