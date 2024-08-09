WITH
  fe AS (
    DELETE FROM feed_entry AS fe
    WHERE
      NOT EXISTS (
        SELECT
          1
        FROM
          profile_feed_entry AS pfe
        WHERE
          pfe.feed_entry_id = fe.id
      )
  ),
  e AS (
    DELETE FROM entry AS e
    WHERE
      NOT EXISTS (
        SELECT
          1
        FROM
          feed_entry AS fe
        WHERE
          fe.entry_id = e.id
      )
  )
DELETE FROM feed AS f
WHERE
  NOT EXISTS (
    SELECT
      1
    FROM
      profile_feed AS pf
    WHERE
      pf.feed_id = f.id
  );