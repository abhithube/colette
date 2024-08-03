UPDATE tags AS t
SET
  title = coalesce($3, t.title)
WHERE
  id = $1
  AND profile_id = $2
RETURNING
  id,
  title;