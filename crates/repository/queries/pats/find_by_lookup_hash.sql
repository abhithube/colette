SELECT
  id,
  verification_hash,
  user_id
FROM
  personal_access_tokens
WHERE
  lookup_hash = $1
