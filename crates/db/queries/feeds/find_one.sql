WITH
  pft AS (
    SELECT
      t.id,
      t.title,
      pft.profile_feed_id
    FROM
      profile_feed_tags AS pft
      INNER JOIN tags AS t ON t.id = pft.tag_id
    ORDER BY
      t.title ASC
  )
SELECT
  pf.id,
  f.link,
  pf.title,
  f.title AS original_title,
  f.url,
  coalesce(
    json_agg(
      DISTINCT jsonb_build_object(
        'id',
        pft.id,
        'title',
        pft.title,
        'bookmark_count',
        NULL::int8,
        'feed_count',
        NULL::int8
      )
    ) FILTER (
      WHERE
        pft.id IS NOT NULL
    ),
    '[]'
  ) AS "tags!: Json<Vec<Tag>>",
  count(pfe.id) AS unread_count
FROM
  profile_feeds AS pf
  INNER JOIN feeds AS f ON f.id = pf.feed_id
  LEFT JOIN pft ON pft.profile_feed_id = pf.id
  LEFT JOIN profile_feed_entries AS pfe ON pfe.profile_feed_id = pf.id
  AND pfe.has_read = FALSE
WHERE
  pf.id = $1
  AND pf.profile_id = $2
GROUP BY
  pf.id,
  f.link,
  f.title,
  f.url;