UPDATE bookmarks
SET
  archived_path = $2
WHERE
  id = $1
