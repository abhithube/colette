DELETE FROM profile_bookmarks
WHERE
  id = $1
  AND profile_id = $2;