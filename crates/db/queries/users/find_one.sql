SELECT
  id,
  email,
  password
FROM
  "user"
WHERE
  id = $1;