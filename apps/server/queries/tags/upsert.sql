WITH
  new_tag AS (
    INSERT INTO
      tags (title, user_id)
    VALUES
      ($1, $2)
    ON CONFLICT (user_id, title) DO NOTHING
    RETURNING
      id
  )
SELECT
  id AS "id!"
FROM
  new_tag
UNION ALL
SELECT
  id
FROM
  tags
WHERE
  user_id = $2
  AND title = $1
