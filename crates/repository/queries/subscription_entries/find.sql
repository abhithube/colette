SELECT
  se.id,
  se.has_read,
  se.read_at,
  se.subscription_id,
  se.feed_entry_id,
  fe.link,
  fe.title,
  fe.published_at,
  fe.description,
  fe.author,
  fe.thumbnail_url,
  fe.feed_id,
  s.user_id
FROM
  subscription_entries se
  INNER JOIN feed_entries fe ON fe.id = se.feed_entry_id
  INNER JOIN subscriptions s ON s.id = se.subscription_id
WHERE
  (
    $1::UUID IS NULL
    OR se.id = $1
  )
  AND (
    $2::UUID IS NULL
    OR s.user_id = $2
  )
  AND (
    $3::UUID IS NULL
    OR se.subscription_id = $3
  )
  AND (
    $4::BOOL IS NULL
    OR se.has_read = $4
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
