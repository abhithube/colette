WITH
  fe AS (
    INSERT INTO
      feed_entries (feed_id, entry_id)
    VALUES
      ($1, $2)
    ON CONFLICT (feed_id, entry_id) DO nothing
    RETURNING
      id
  )
SELECT
  id "id!"
FROM
  fe
UNION ALL
SELECT
  id
FROM
  feed_entries
WHERE
  feed_id = $1
  AND entry_id = $2;