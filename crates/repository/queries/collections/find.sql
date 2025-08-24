SELECT
  id,
  title,
  filter_json AS "filter_json: Json<BookmarkFilter>",
  created_at,
  updated_at
FROM
  collections
WHERE
  user_id = $1
  AND (
    $2::UUID IS NULL
    OR id = $2
  )
  AND (
    $3::TEXT IS NULL
    OR title > $3
  )
ORDER BY
  title ASC
LIMIT
  $4
