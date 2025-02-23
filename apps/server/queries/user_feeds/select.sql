WITH
  unread_counts AS (
    SELECT
      uf.id AS uf_id,
      count(ufe.id) AS count
    FROM
      user_feeds uf
      INNER JOIN user_feed_entries ufe ON ufe.user_feed_id = uf.id
      AND NOT ufe.has_read
    GROUP BY
      uf.id
  ),
  json_tags AS (
    SELECT
      uf.id AS uf_id,
      jsonb_agg(
        jsonb_build_object('id', t.id, 'title', t.title)
        ORDER BY
          t.title
      ) FILTER (
        WHERE
          t.id IS NOT NULL
      ) AS tags
    FROM
      user_feeds uf
      INNER JOIN user_feed_tags uft ON uft.user_feed_id = uf.id
      INNER JOIN tags t ON t.id = uft.tag_id
    GROUP BY
      uf.id
  )
SELECT
  uf.id,
  uf.title,
  f.link AS "link: DbUrl",
  f.xml_url AS "xml_url: DbUrl",
  uf.created_at,
  uf.updated_at,
  coalesce(jt.tags, '[]'::jsonb) AS "tags: Json<Vec<Tag>>",
  coalesce(uc.count, 0) AS unread_count
FROM
  user_feeds uf
  INNER JOIN feeds f ON f.id = uf.feed_id
  LEFT JOIN json_tags jt ON jt.uf_id = uf.id
  LEFT JOIN unread_counts uc ON uc.uf_id = uf.id
WHERE
  uf.user_id = $1
  AND (
    $2::BOOLEAN
    OR uf.id = $3
  )
  AND (
    $4::BOOLEAN
    OR EXISTS (
      SELECT
        t.*
      FROM
        jsonb_array_elements(jt.tags) t
      WHERE
        t ->> 'title' = ANY ($5)
    )
  )
  AND (
    $6::BOOLEAN
    OR (uf.title, uf.id) > ($7, $8)
  )
ORDER BY
  uf.title ASC,
  uf.id ASC
LIMIT
  $9
