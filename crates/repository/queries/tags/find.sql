SELECT
  id,
  title,
  user_id,
  created_at,
  updated_at
FROM
  tags
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
