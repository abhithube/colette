SELECT
  id,
  email,
  password
FROM
  "user"
WHERE
  email = $1;