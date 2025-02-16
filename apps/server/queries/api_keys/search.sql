SELECT
  verification_hash,
  user_id
FROM
  api_keys
WHERE
  lookup_hash = $1
