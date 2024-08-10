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
    $6::TEXT[] IS NULL
    OR pfe.profile_feed_id IN (
      SELECT DISTINCT
        pft.profile_feed_id
      FROM
        profile_feed_tag pft
        INNER JOIN tag AS t ON t.id = pft.tag_id
      WHERE
        t.title = ANY ($6)
    )
  )
ORDER BY
  e.published_at DESC,
  e.title ASC,
  pfe.id DESC
LIMIT
  $2;