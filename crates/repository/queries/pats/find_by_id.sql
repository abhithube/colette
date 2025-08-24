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
  personal_access_tokens
WHERE
  id = $1
  AND user_id = $2
