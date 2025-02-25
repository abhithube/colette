WITH
  unread_count AS (
    SELECT
      ufe.user_feed_id,
      coalesce(count(ufe.id), 0) AS count
    FROM
      user_feed_entries ufe
    WHERE
      NOT ufe.has_read
    GROUP BY
      ufe.user_feed_id
  ),
  json_tags AS (
    SELECT
      uft.user_feed_id,
      coalesce(
        jsonb_agg(
          jsonb_build_object('id', t.id, 'title', t.title)
          ORDER BY
            t.title
        ),
        '[]'::jsonb
      ) AS tags
    FROM
      user_feed_tags uft
      INNER JOIN tags t ON t.id = uft.tag_id
    GROUP BY
      uft.user_feed_id
  )
SELECT
  uf.id,
  uf.title,
  f.link AS "link: DbUrl",
  f.xml_url AS "xml_url: DbUrl",
  uf.created_at,
  uf.updated_at,
  jt.tags AS "tags: Json<Vec<Tag>>",
  uc.count AS unread_count
FROM
  user_feeds uf
  INNER JOIN feeds f ON f.id = uf.feed_id
  LEFT JOIN json_tags jt ON jt.user_feed_id = uf.id
  LEFT JOIN unread_count uc ON uc.user_feed_id = uf.id
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
        1
      FROM
        user_feed_tags uft
        JOIN tags t ON t.id = uft.tag_id
      WHERE
        uft.user_feed_id = uf.id
        AND t.title = ANY ($5)
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
