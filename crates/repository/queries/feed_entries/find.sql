SELECT
  id,
  link AS "link: DbUrl",
  title,
  published_at,
  description,
  author,
  thumbnail_url AS "thumbnail_url: DbUrl",
  feed_id
FROM
  feed_entries
WHERE
  (
    $1::UUID IS NULL
    OR id = $1
  )
  AND (
    $2::UUID IS NULL
    OR feed_id = $2
  )
  AND (
    (
      $3::TIMESTAMPTZ IS NULL
      OR $4::UUID IS NULL
    )
    OR (published_at, id) > ($3, $4)
  )
ORDER BY
  published_at DESC,
  id DESC
LIMIT
  $5
