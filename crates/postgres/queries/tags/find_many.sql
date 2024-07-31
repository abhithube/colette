SELECT
  id,
  title
FROM
  tags
WHERE
  profile_id = $1;