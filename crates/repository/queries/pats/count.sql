SELECT
  count(id)
FROM
  personal_access_tokens
WHERE
  user_id = $1
