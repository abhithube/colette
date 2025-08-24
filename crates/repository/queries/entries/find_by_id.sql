SELECT
  fe.id,
  rs.created_at AS read_at,
  rs.user_id
FROM
  feed_entries fe
  LEFT JOIN read_statuses rs ON rs.feed_entry_id = fe.id
WHERE
  fe.id = $1
  AND rs.user_id = $2
