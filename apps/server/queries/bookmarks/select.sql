WITH
  json_tags AS (
    SELECT
      bt.bookmark_id,
      jsonb_agg(
        jsonb_build_object('id', t.id, 'title', t.title)
        ORDER BY
          t.title
      ) AS tags
    FROM
      bookmark_tags bt
      JOIN tags t ON t.id = bt.tag_id
    WHERE
      bt.user_id = $1
    GROUP BY
      bt.bookmark_id
  )
SELECT
  b.id,
  b.link AS "link: DbUrl",
  b.title,
  b.thumbnail_url AS "thumbnail_url: DbUrl",
  b.published_at,
  b.author,
  b.archived_path,
  b.created_at,
  b.updated_at,
  coalesce(jt.tags, '[]'::jsonb) AS "tags: Json<Vec<Tag>>"
FROM
  bookmarks b
  LEFT JOIN json_tags jt ON jt.bookmark_id = b.id
WHERE
  b.user_id = $1
  AND (
    $2::BOOLEAN
    OR b.id = $3
  )
  AND (
    $4::BOOLEAN
    OR EXISTS (
      SELECT
        1
      FROM
        bookmark_tags bt
        JOIN tags t ON t.id = bt.tag_id
      WHERE
        bt.bookmark_id = b.id
        AND t.title = ANY ($5)
    )
  )
  AND (
    $6::BOOLEAN
    OR b.created_at > $7
  )
ORDER BY
  b.created_at ASC
LIMIT
  $8
