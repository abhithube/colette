WITH
  pf AS (
    SELECT
      pf.id,
      pf.profile_id,
      f.link,
      pf.title,
      f.title AS original_title,
      f.url,
      count(pfe.id) AS unread_count
    FROM
      profile_feeds AS pf
      INNER JOIN feeds AS f ON f.id = pf.feed_id
      LEFT JOIN profile_feed_entries AS pfe ON pfe.profile_feed_id = pf.id
      AND pfe.has_read = FALSE
    WHERE
      pf.id = $1
      AND pf.profile_id = $2
    GROUP BY
      pf.id,
      f.link,
      f.title,
      f.url
  ),
  t_insert AS (
    INSERT INTO
      tags (title, profile_id)
    SELECT
      unnest($3::TEXT[]),
      pf.profile_id
    FROM
      pf
    ON CONFLICT (profile_id, title) DO nothing
    RETURNING
      id,
      title
  ),
  t AS (
    SELECT
      id,
      title
    FROM
      t_insert
    UNION ALL
    SELECT
      t.id,
      t.title
    FROM
      tags t,
      pf
    WHERE
      t.title = ANY ($3::TEXT[])
      AND t.profile_id = pf.profile_id
  ),
  pft_insert AS (
    INSERT INTO
      profile_feed_tags (profile_feed_id, tag_id, profile_id)
    SELECT
      pf.id,
      t.id,
      pf.profile_id
    FROM
      pf,
      t
    ON CONFLICT DO nothing
    RETURNING
      profile_feed_id,
      tag_id
  ),
  pft_delete AS (
    DELETE FROM profile_feed_tags USING pf
    WHERE
      profile_feed_id = pf.id
      AND tag_id NOT IN (
        SELECT
          t.id
        FROM
          t
      )
  ),
  pft AS (
    SELECT
      profile_feed_id,
      tag_id
    FROM
      pft_insert
    UNION ALL
    SELECT
      pft.profile_feed_id,
      pft.tag_id
    FROM
      profile_feed_tags pft,
      pf
    WHERE
      pft.profile_feed_id = pf.id
  )
SELECT
  pf.id,
  pf.link,
  pf.title,
  pf.original_title,
  pf.url,
  coalesce(
    array_agg(
      DISTINCT ROW (t.id, t.title, NULL::int8, NULL::int8)
    ) FILTER (
      WHERE
        t.id IS NOT NULL
    ),
    ARRAY[]::record[]
  ) AS "tags!: Vec<Tag>",
  pf.unread_count
FROM
  pf
  LEFT JOIN pft ON pft.profile_feed_id = pf.id
  LEFT JOIN t ON t.id = pft.tag_id
GROUP BY
  pf.id,
  pf.link,
  pf.title,
  pf.original_title,
  pf.url,
  pf.unread_count;