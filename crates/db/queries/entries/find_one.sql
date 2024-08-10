SELECT
  pfe.id,
  e.link,
  e.title,
  e.published_at AS "published_at: DateTime<Utc>",
  e.description,
  e.author,
  e.thumbnail_url,
  pfe.has_read,
  pfe.profile_feed_id AS feed_id
FROM
  profile_feed_entry AS pfe
  INNER JOIN feed_entry AS fe ON fe.id = pfe.feed_entry_id
  INNER JOIN entry AS e ON e.id = fe.entry_id
WHERE
  pfe.id = $1
  AND pfe.profile_id = $2;