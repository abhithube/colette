DELETE FROM tag
WHERE
  id = $1
  AND profile_id = $2;