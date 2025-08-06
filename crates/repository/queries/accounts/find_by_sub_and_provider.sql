SELECT
  id,
  password_hash,
  user_id
FROM
  accounts
WHERE
  sub = $1
  AND provider = $2
