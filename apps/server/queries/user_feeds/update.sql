UPDATE user_feeds
SET
  title = CASE
    WHEN $3 THEN $4
    ELSE title
  END
WHERE
  id = $1
  AND user_id = $2
