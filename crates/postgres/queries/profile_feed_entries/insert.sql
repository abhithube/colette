WITH
  pfe AS (
    INSERT INTO
      profile_feed_entries (profile_feed_id, feed_entry_id)
    VALUES
      ($1, $2)
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
  profile_feed_id = $1
  AND feed_entry_id = $2;
