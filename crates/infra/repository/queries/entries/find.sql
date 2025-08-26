SELECT
  fe.id,
  fe.link,
  fe.title,
  fe.published_at,
  fe.description,
  fe.author,
  fe.thumbnail_url,
  rs.created_at AS read_at,
  fe.feed_id
FROM
  feed_entries fe
  LEFT JOIN read_statuses rs ON rs.feed_entry_id = fe.id
  INNER JOIN feeds f ON f.id = fe.feed_id
  INNER JOIN subscriptions s ON s.feed_id = f.id
WHERE
  s.user_id = $1
  AND (
    $2::UUID IS NULL
    OR fe.id = $2
  )
  AND (
    $3::UUID IS NULL
    OR s.id = $3
  )
  AND (
    $4::BOOL IS NULL
    OR rs.feed_entry_id IS NOT NULL
  )
  AND (
    $5::UUID[] IS NULL
    OR EXISTS (
      SELECT
        1
      FROM
        subscription_tags st
      WHERE
        st.subscription_id = s.id
        AND st.tag_id = ANY ($5)
    )
  )
  AND (
    (
      $6::TIMESTAMPTZ IS NULL
      OR $7::UUID IS NULL
    )
    OR (fe.published_at, fe.id) > ($6, $7)
  )
ORDER BY
  fe.published_at DESC,
  fe.id DESC
LIMIT
  $8
