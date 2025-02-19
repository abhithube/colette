WITH
  json_tags AS (
    SELECT
      b.id AS bookmark_id,
      jsonb_agg(
        jsonb_build_object('id', t.id, 'title', t.title)
        ORDER BY
          t.title
      ) FILTER (
        WHERE
          t.id IS NOT NULL
      ) AS tags
    FROM
      bookmarks b
      INNER JOIN bookmark_tags bt ON bt.bookmark_id = b.id
      INNER JOIN tags t ON t.id = bt.tag_id
    GROUP BY
      b.id
  )
SELECT
  b.id,
  b.link AS "link: DbUrl",
  b.title,
  b.thumbnail_url AS "thumbnail_url: DbUrl",
  b.published_at,
  b.author,
  b.archived_path,
  b.collection_id,
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
    OR CASE
      WHEN $5::uuid IS NULL THEN b.collection_id IS NULL
      ELSE b.collection_id = $5
    END
  )
  AND (
    $6::BOOLEAN
    OR EXISTS (
      SELECT
        1
      FROM
        jsonb_array_elements(jt.tags) t
      WHERE
        t ->> 'title' = ANY ($7)
    )
  )
  AND (
    $8::BOOLEAN
    OR b.created_at > $9
  )
ORDER BY
  b.created_at ASC
LIMIT
  $10
