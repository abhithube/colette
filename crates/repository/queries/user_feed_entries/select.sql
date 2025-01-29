SELECT
  ufe.id,
  ufe.has_read,
  ufe.user_feed_id AS feed_id,
  fe.link,
  fe.title,
  fe.published_at,
  fe.description,
  fe.author,
  fe.thumbnail_url
FROM
  user_feed_entries ufe
  JOIN feed_entries fe ON fe.id = ufe.feed_entry_id
  LEFT JOIN user_feed_tags uft ON $1
  AND uft.user_feed_id = ufe.user_feed_id
  LEFT JOIN tags t ON $1
  AND t.id = uft.tag_id
  AND t.title = ANY ($2)
WHERE
  NOT $1
  OR t.id IS NOT NULL
  AND ufe.user_id = $3
  AND (
    $4::BOOLEAN
    OR ufe.id = $5
  )
  AND (
    $6::BOOLEAN
    OR ufe.user_feed_id = $7
  )
  AND (
    $8::BOOLEAN
    OR ufe.has_read = $9
  )
  AND (
    $10::BOOLEAN
    OR (fe.published_at, ufe.id) > ($11, $12)
  )
ORDER BY
  fe.published_at DESC,
  ufe.id DESC
LIMIT
  $13
