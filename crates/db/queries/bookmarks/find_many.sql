SELECT
  pb.id,
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
  profile_bookmarks AS pb
  INNER JOIN bookmarks AS b ON b.id = pb.bookmark_id
  LEFT JOIN profile_bookmark_tags AS pbt ON pbt.profile_bookmark_id = pb.id
  LEFT JOIN tags AS t ON pbt.tag_id = t.id
WHERE
  pb.profile_id = $1
  AND (
    $3::UUID [] IS NULL
    OR pb.id IN (
      SELECT DISTINCT
        profile_bookmark_id
      FROM
        profile_bookmark_tags
      WHERE
        tag_id = ANY ($3)
    )
  )
GROUP BY
  pb.id,
  b.link,
  b.title,
  b.thumbnail_url,
  b.published_at,
  b.author
ORDER BY
  b.published_at DESC,
  b.title ASC,
  pb.id ASC
LIMIT
  $2;