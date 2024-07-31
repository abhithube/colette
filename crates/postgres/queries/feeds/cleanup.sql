WITH
  fe AS (
    DELETE FROM feed_entries AS fe
    WHERE
      NOT EXISTS (
        SELECT
          1
        FROM
          profile_feed_entries AS pfe
        WHERE
          pfe.feed_entry_id = fe.id
      )
  ),
  e AS (
    DELETE FROM entries AS e
    WHERE
      NOT EXISTS (
        SELECT
          1
        FROM
          feed_entries AS fe
        WHERE
          fe.entry_id = e.id
      )
  )
DELETE FROM feeds AS f
WHERE
  NOT EXISTS (
    SELECT
      1
    FROM
      profile_feeds AS pf
    WHERE
      pf.feed_id = f.id
  );