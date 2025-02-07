UPDATE user_feed_entries
SET
  has_read = CASE
    WHEN $3 THEN $4
    ELSE has_read
  END
WHERE
  id = $1
  AND user_id = $2
