SELECT
  id,
  title,
  preview,
  created_at
FROM
  api_keys
WHERE
  user_id = $1
  AND (
    $2::BOOLEAN
    OR id = $3
  )
  AND (
    $4::BOOLEAN
    OR created_at > $5
  )
ORDER BY
  created_at ASC
LIMIT
  $6
