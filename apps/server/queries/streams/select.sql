SELECT
  id,
  title,
  "filter" AS "filter: Json<Value>",
  created_at,
  updated_at
FROM
  streams
WHERE
  user_id = $1
  AND (
    $2::BOOLEAN
    OR id = $3
  )
  AND (
    $4::BOOLEAN
    OR title > $5
  )
ORDER BY
  title ASC
LIMIT
  $6
