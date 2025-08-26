SELECT
  id,
  title,
  preview,
  created_at,
  updated_at
FROM
  personal_access_tokens
WHERE
  user_id = $1
  AND (
    $2::UUID IS NULL
    OR id = $2
  )
  AND (
    $3::TIMESTAMPTZ IS NULL
    OR created_at > $3
  )
ORDER BY
  created_at ASC
LIMIT
  $4
