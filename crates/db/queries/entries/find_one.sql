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
  pfe.id = $1
  AND pfe.profile_id = $2;