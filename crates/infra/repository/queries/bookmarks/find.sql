SELECT
  b.id,
  b.link AS "link: DbUrl",
  b.title,
  b.thumbnail_url AS "thumbnail_url: DbUrl",
  b.published_at,
  b.author,
  b.archived_path,
  coalesce(bt.tags, '[]'::JSONB) AS "tags: Json<Vec<TagRow>>",
  b.created_at,
  b.updated_at
FROM
  bookmarks b
  LEFT JOIN (
    SELECT
      bt.bookmark_id,
      jsonb_agg(
        jsonb_build_object(
          'id',
          t.id,
          'title',
          t.title,
          'user_id',
          t.user_id,
          'created_at',
          t.created_at,
          'updated_at',
          t.updated_at
        )
        ORDER BY
          t.title ASC
      ) AS tags
    FROM
      bookmark_tags bt
      INNER JOIN tags t ON t.id = bt.tag_id
    WHERE
      t.user_id = $1
      AND (
        $2::UUID IS NULL
        OR bt.bookmark_id = $2
      )
    GROUP BY
      bt.bookmark_id
  ) AS bt ON bt.bookmark_id = b.id
WHERE
  user_id = $1
  AND (
    $2::UUID IS NULL
    OR id = $2
  )
  AND (
    $3::UUID[] IS NULL
    OR EXISTS (
      SELECT
        1
      FROM
        bookmark_tags bt
      WHERE
        bt.bookmark_id = b.id
        AND bt.tag_id = ANY ($3)
    )
  )
  AND (
    $4::TIMESTAMPTZ IS NULL
    OR b.created_at > $4
  )
ORDER BY
  b.created_at DESC
LIMIT
  $5
