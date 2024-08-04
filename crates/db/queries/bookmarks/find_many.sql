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
  b.profile_id = $1
  AND (
    $3::UUID [] IS NULL
    OR b.id IN (
      SELECT DISTINCT
        bookmark_id
      FROM
        bookmark_tags
      WHERE
        tag_id = ANY ($3)
    )
  )
GROUP BY
  b.id,
  b.link,
  b.title,
  b.thumbnail_url,
  b.published_at,
  b.author
ORDER BY
  b.published_at DESC,
  b.title ASC,
  b.id ASC
LIMIT
  $2;