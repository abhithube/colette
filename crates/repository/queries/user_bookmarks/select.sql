WITH
  json_tags AS (
    SELECT
      ub.id AS ub_id,
      jsonb_agg(
        jsonb_build_object('id', t.id, 'title', t.title)
        ORDER BY
          t.title
      ) FILTER (
        WHERE
          t.id IS NOT NULL
      ) AS tags
    FROM
      user_bookmarks ub
      INNER JOIN user_bookmark_tags ubt ON ubt.user_bookmark_id = ub.id
      INNER JOIN tags t ON t.id = ubt.tag_id
    GROUP BY
      ub.id
  )
SELECT
  ub.id,
  ub.title,
  ub.thumbnail_url,
  ub.published_at,
  ub.author,
  ub.folder_id,
  ub.created_at,
  b.link,
  b.title AS original_title,
  b.thumbnail_url AS original_thumbnail_url,
  b.published_at AS original_published_at,
  b.author AS original_author,
  jt.tags AS "tags: Json<Vec<Tag>>"
FROM
  user_bookmarks ub
  INNER JOIN bookmarks b ON b.id = ub.bookmark_id
  LEFT JOIN json_tags jt ON jt.ub_id = ub.id
WHERE
  ub.user_id = $1
  AND (
    $2::BOOLEAN
    OR ub.id = $3
  )
  AND (
    $4::BOOLEAN
    OR CASE
      WHEN $5::uuid IS NULL THEN ub.folder_id IS NULL
      ELSE ub.folder_id = $5
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
    OR coalesce(ub.created_at, b.created_at) > $9
  )
ORDER BY
  coalesce(ub.published_at, b.created_at) ASC
LIMIT
  $10
