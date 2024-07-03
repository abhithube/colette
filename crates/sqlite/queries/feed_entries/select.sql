SELECT
  id
FROM
  feed_entries
WHERE
  feed_id = ?1
  AND entry_id = ?2;