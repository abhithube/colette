SELECT
  pf.id,
  f.link,
  f.title,
  f.url,
  coalesce(
    array_agg(ROW (t.id, t.title, NULL::int8, NULL::int8)) FILTER (
      WHERE
        t.id IS NOT NULL
    ),
    ARRAY[]::record[]
  ) AS "tags!: Vec<Tag>",
  count(pfe.id) AS unread_count
FROM
  profile_feeds AS pf
  INNER JOIN feeds AS f ON f.id = pf.feed_id
  LEFT JOIN profile_feed_tags AS pft ON pft.profile_feed_id = pf.id
  LEFT JOIN tags AS t ON t.id = pft.tag_id
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