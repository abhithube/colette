DELETE FROM read_statuses
WHERE
  feed_entry_id = $1
  AND user_id = $2
