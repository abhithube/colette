SELECT
  pfe.id,
  e.link,
  e.title,
  e.published_at,
  e.description,
  e.author,
  e.thumbnail_url,
  pfe.has_read,
  pfe.profile_feed_id AS feed_id
FROM
  profile_feed_entries AS pfe
  INNER JOIN feed_entries AS fe ON fe.id = pfe.feed_entry_id
  INNER JOIN entries AS e ON e.id = fe.entry_id
WHERE
  pfe.profile_id = $1
  AND (
    $3::timestamptz IS NULL
    OR e.published_at < $3
  )
  AND (
    $4::UUID IS NULL
    OR pfe.profile_feed_id = $4
  )
  AND (
    $5::boolean IS NULL
    OR pfe.has_read = $5
  )
  AND (
    $6::UUID [] IS NULL
    OR pfe.profile_feed_id IN (
      SELECT DISTINCT
        profile_feed_id
      FROM
        profile_feed_tags
      WHERE
        tag_id = ANY ($6)
    )
  )
ORDER BY
  e.published_at DESC,
  e.title ASC,
  pfe.id DESC
LIMIT
  $2;