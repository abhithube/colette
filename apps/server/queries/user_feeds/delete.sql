DELETE FROM user_feeds
WHERE
  id = $1
  AND user_id = $2
