WITH
  pbt AS (
    SELECT
      t.id,
      t.title,
      pbt.profile_bookmark_id
    FROM
      profile_bookmark_tags AS pbt
      INNER JOIN tags AS t ON t.id = pbt.tag_id
    ORDER BY
      t.title ASC
  )
SELECT
  pb.id,
  b.link,
  b.title,
  b.thumbnail_url,
  b.published_at,
  b.author,
  coalesce(
    json_agg(
      DISTINCT jsonb_build_object(
        'id',
        pbt.id,
        'title',
        pbt.title,
        'bookmark_count',
        NULL::int8,
        'feed_count',
        NULL::int8
      )
    ) FILTER (
      WHERE
        pbt.id IS NOT NULL
    ),
    '[]'
  ) AS "tags!: Json<Vec<Tag>>"
FROM
  profile_bookmarks AS pb
  INNER JOIN bookmarks AS b ON b.id = pb.bookmark_id
  LEFT JOIN pbt ON pbt.profile_bookmark_id = pb.id
WHERE
  pb.id = $1
  AND pb.profile_id = $2
GROUP BY
  pb.id,
  b.link,
  b.title,
  b.thumbnail_url,
  b.published_at,
  b.author;