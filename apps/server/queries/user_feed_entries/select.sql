SELECT
  ufe.id,
  ufe.has_read,
  ufe.user_feed_id AS feed_id,
  fe.link AS "link: DbUrl",
  fe.title,
  fe.published_at,
  fe.description,
  fe.author,
  fe.thumbnail_url AS "thumbnail_url: DbUrl",
  ufe.created_at,
  ufe.updated_at
FROM
  user_feed_entries ufe
  JOIN feed_entries fe ON fe.id = ufe.feed_entry_id
WHERE
  ufe.user_id = $1
  AND (
    $2::BOOLEAN
    OR ufe.id = $3
  )
  AND (
    $4::BOOLEAN
    OR ufe.user_feed_id = $5
  )
  AND (
    $6::BOOLEAN
    OR ufe.has_read = $7
  )
  AND (
    $8::BOOLEAN
    OR EXISTS (
      SELECT
        1
      FROM
        user_feed_tags uft
        JOIN tags t ON t.id = uft.tag_id
      WHERE
        uft.user_feed_id = ufe.user_feed_id
        AND t.title = ANY ($9)
    )
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
