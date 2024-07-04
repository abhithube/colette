WITH
  pfe AS (
    INSERT INTO
      profile_feed_entries (id, profile_feed_id, feed_entry_id)
    VALUES
      ($1, $2, $3)
    ON CONFLICT (profile_feed_id, feed_entry_id) DO nothing
    RETURNING
      id
  )
SELECT
  id "id!"
FROM
  pfe
UNION ALL
SELECT
  id
FROM
  profile_feed_entries
WHERE
  profile_feed_id = $2
  AND feed_entry_id = $3;
