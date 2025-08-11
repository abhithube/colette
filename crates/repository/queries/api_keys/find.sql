SELECT
  id,
  lookup_hash,
  verification_hash,
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
    $2::TEXT IS NULL
    OR lookup_hash = $2
  )
  AND (
    $3::UUID IS NULL
    OR user_id = $3
  )
  AND (
    $4::TIMESTAMPTZ IS NULL
    OR created_at > $4
  )
ORDER BY
  created_at ASC
LIMIT
  $5
