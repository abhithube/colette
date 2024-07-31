WITH
  f AS (
    INSERT INTO
      feeds (link, title, url)
    VALUES
      ($2, $3, $4)
    ON CONFLICT (link) DO
    UPDATE
    SET
      title = excluded.title,
      url = excluded.url
    RETURNING
      id,
      link,
      title,
      url
  ),
  pf_insert AS (
    INSERT INTO
      profile_feeds (profile_id, feed_id)
    SELECT
      $1,
      f.id
    FROM
      f
    ON CONFLICT (profile_id, feed_id) DO nothing
    RETURNING
      id,
      profile_id,
      feed_id
  ),
  pf AS (
    SELECT
      id AS "id!",
      profile_id,
      feed_id
    FROM
      pf_insert
    UNION ALL
    SELECT
      pf.id,
      pf.profile_id,
      pf.feed_id
    FROM
      profile_feeds pf,
      f
    WHERE
      pf.profile_id = $1
      AND pf.feed_id = f.id
  ),
  e AS (
    INSERT INTO
      entries (
        link,
        title,
        published_at,
        description,
        author,
        thumbnail_url
      )
    SELECT
      *
    FROM
      unnest(
        $5::TEXT[],
        $6::TEXT[],
        $7::TIMESTAMPTZ[],
        $8::TEXT[],
        $9::TEXT[],
        $10::TEXT[]
      )
    ON CONFLICT (link) DO
    UPDATE
    SET
      title = excluded.title,
      published_at = excluded.published_at,
      description = excluded.description,
      author = excluded.author,
      thumbnail_url = excluded.thumbnail_url
    RETURNING
      id
  ),
  fe_insert AS (
    INSERT INTO
      feed_entries (feed_id, entry_id)
    SELECT
      f.id,
      e.id
    FROM
      f,
      e
    ON CONFLICT (feed_id, entry_id) DO nothing
    RETURNING
      id
  ),
  fe AS (
    SELECT
      id
    FROM
      fe_insert
    UNION ALL
    SELECT
      fe.id
    FROM
      feed_entries fe,
      f,
      e
    WHERE
      fe.feed_id = f.id
      AND fe.entry_id IN (
        SELECT
          id
        FROM
          e
      )
  ),
  pfe AS (
    INSERT INTO
      profile_feed_entries (profile_feed_id, feed_entry_id, profile_id)
    SELECT
      pf."id!",
      fe.id,
      pf.profile_id
    FROM
      pf,
      fe
    ON CONFLICT (profile_feed_id, feed_entry_id) DO nothing
    RETURNING
      id
  )
SELECT
  pf."id!",
  f.link,
  f.title,
  f.url,
  coalesce(
    array_agg(ROW (t.id, t.title)) FILTER (
      WHERE
        t.id IS NOT NULL
    ),
    ARRAY[]::record[]
  ) AS "tags!: Vec<Tag>",
  count(pfe.id) AS unread_count
FROM
  pf
  JOIN f ON f.id = pf.feed_id
  LEFT JOIN profile_feed_tags AS pft ON pft.profile_feed_id = pf."id!"
  LEFT JOIN tags AS t ON pft.tag_id = t.id
  LEFT JOIN profile_feed_entries AS pfe ON pfe.profile_feed_id = pf."id!"
  AND pfe.has_read = FALSE
GROUP BY
  pf."id!",
  f.link,
  f.title,
  f.url;