WITH
  new_tags AS (
    INSERT INTO
      tags (title, user_id)
    SELECT
      unnest($1::TEXT[]),
      $2
    ON CONFLICT (user_id, title) DO NOTHING
    RETURNING
      id
  ),
  all_tags AS (
    SELECT
      id
    FROM
      new_tags
    UNION ALL
    SELECT
      id
    FROM
      tags
    WHERE
      user_id = $2
      AND title = ANY ($1::TEXT[])
  ),
  deleted_uft AS (
    DELETE FROM user_feed_tags uft
    WHERE
      uft.user_id = $2
      AND uft.user_feed_id = $3
      AND uft.tag_id NOT IN (
        SELECT
          id
        FROM
          all_tags
      )
  )
INSERT INTO
  user_feed_tags (user_feed_id, tag_id, user_id)
SELECT
  $3,
  all_tags.id,
  $2
FROM
  all_tags
ON CONFLICT (user_feed_id, tag_id) DO NOTHING
