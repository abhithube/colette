SELECT
  pfe.id,
  e.link,
  e.title,
  e.published_at,
  e.description,
  e.author,
  e.thumbnail_url,
  pfe.has_read,
  pfe.profile_feed_id feed_id
FROM
  profile_feed_entries pfe
  JOIN profile_feeds pf ON pf.id = pfe.profile_feed_id
  JOIN feed_entries fe ON fe.id = pfe.feed_entry_id
  JOIN entries e ON e.id = fe.entry_id
WHERE
  pf.profile_id = $1
  AND (
    $3::timestamptz IS NULL
    OR e.published_at < $3
  )
  AND (
    $4::text IS NULL
    OR pfe.profile_feed_id = $4
  )
  AND (
    $5::boolean IS NULL
    OR pfe.has_read = $5
  )
ORDER BY
  e.published_at DESC,
  pfe.id DESC
LIMIT
  $2;
