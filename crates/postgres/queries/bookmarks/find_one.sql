SELECT
  b.id,
  b.link,
  b.title,
  b.thumbnail_url,
  b.published_at,
  b.author,
  coalesce(
    array_agg(ROW (t.id, t.title)) FILTER (
      WHERE
        t.id IS NOT NULL
    ),
    ARRAY[]::record[]
  ) AS "tags!: Vec<Tag>"
FROM
  bookmarks b
  LEFT JOIN bookmark_tags AS bt ON bt.bookmark_id = b.id
  LEFT JOIN tags AS t ON bt.tag_id = t.id
WHERE
  b.id = $1
  AND b.profile_id = $2
GROUP BY
  b.id,
  b.link,
  b.title,
  b.thumbnail_url,
  b.published_at,
  b.author;