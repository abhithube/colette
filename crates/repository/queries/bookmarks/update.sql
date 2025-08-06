UPDATE bookmarks
SET
  title = CASE
    WHEN $2::TEXT IS NOT NULL THEN $2
    ELSE bookmarks.title
  END,
  thumbnail_url = CASE
    WHEN $3 THEN $4
    ELSE bookmarks.thumbnail_url
  END,
  published_at = CASE
    WHEN $5 THEN $6
    ELSE bookmarks.published_at
  END,
  author = CASE
    WHEN $7 THEN $8
    ELSE bookmarks.author
  END
WHERE
  id = $1
