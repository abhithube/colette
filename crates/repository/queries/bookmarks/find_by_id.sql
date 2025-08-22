SELECT
  b.id,
  b.link AS "link: DbUrl",
  b.title,
  b.thumbnail_url AS "thumbnail_url: DbUrl",
  b.published_at,
  b.author,
  array_agg(
    bt.tag_id
    ORDER BY
      bt.created_at ASC
  ) AS "tags!",
  b.user_id,
  b.created_at,
  b.updated_at
FROM
  bookmarks b
  LEFT JOIN bookmark_tags bt ON bt.bookmark_id = b.id
WHERE
  b.id = $1
  AND b.user_id = $2
GROUP BY
  b.id
