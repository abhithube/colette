SELECT
  id,
  thumbnail_url AS "thumbnail_url: DbUrl",
  archived_path,
  user_id
FROM
  bookmarks
WHERE
  id = $1
