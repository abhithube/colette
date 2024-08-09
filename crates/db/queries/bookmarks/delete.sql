DELETE FROM profile_bookmark
WHERE
  id = $1
  AND profile_id = $2;