SELECT
  id,
  title,
  preview,
  user_id,
  created_at,
  updated_at
FROM
  api_keys
WHERE
  (
    $1::UUID IS NULL
    OR id = $1
  )
  AND (
    $2::UUID IS NULL
    OR user_id = $2
  )
  AND (
    $3::TIMESTAMPTZ IS NULL
    OR created_at > $3
  )
ORDER BY
  created_at ASC
LIMIT
  $4
