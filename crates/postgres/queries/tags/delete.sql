DELETE FROM tags
WHERE
  id = $1
  AND profile_id = $2;