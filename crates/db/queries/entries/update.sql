WITH
  pfe AS (
    UPDATE profile_feed_entries AS pfe
    SET
      has_read = coalesce($3, pfe.has_read)
    WHERE
      pfe.id = $1
      AND pfe.profile_id = $2
    RETURNING
      pfe.id,
      pfe.has_read,
      pfe.profile_feed_id,
      pfe.feed_entry_id
  )
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
  pfe
  INNER JOIN feed_entries AS fe ON fe.id = pfe.feed_entry_id
  INNER JOIN entries AS e ON e.id = fe.entry_id;