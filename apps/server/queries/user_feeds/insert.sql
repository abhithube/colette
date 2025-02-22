WITH
  new_uf AS (
    INSERT INTO
      user_feeds (title, folder_id, feed_id, user_id)
    VALUES
      ($1, $2, $3, $4)
    RETURNING
      id
  )
SELECT
  id AS "id!"
FROM
  new_uf
UNION ALL
SELECT
  id
FROM
  user_feeds
WHERE
  user_id = $4
  AND feed_id = $3
