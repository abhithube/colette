WITH
  pf AS (
    INSERT INTO
      profile_feeds (id, profile_id, feed_id)
    VALUES
      ($1, $2, $3)
    ON CONFLICT (profile_id, feed_id) DO nothing
    RETURNING
      id
  )
SELECT
  id "id!"
FROM
  pf
UNION ALL
SELECT
  id
FROM
  profile_feeds
WHERE
  profile_id = $2
  AND feed_id = $3;