SELECT
  t.id,
  t.title,
  t.created_at,
  t.updated_at,
  count(uft.user_feed_id) AS feed_count,
  count(bt.bookmark_id) AS bookmark_count
FROM
  tags t
  LEFT JOIN user_feed_tags uft ON uft.tag_id = t.id
  LEFT JOIN bookmark_tags bt ON bt.tag_id = t.id
WHERE
  t.user_id = $1
  AND (
    $2::BOOLEAN
    OR t.id = $3
  )
  AND (
    $4::BOOLEAN
    OR t.title > $5
  )
GROUP BY
  t.id,
  t.title
ORDER BY
  t.title ASC
LIMIT
  $6
