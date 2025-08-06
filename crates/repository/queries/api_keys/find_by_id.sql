SELECT
  id,
  user_id
FROM
  api_keys
WHERE
  id = $1
