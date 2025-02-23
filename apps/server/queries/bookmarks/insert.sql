INSERT INTO
  bookmarks (
    link,
    title,
    thumbnail_url,
    published_at,
    author,
    user_id
  )
VALUES
  ($1, $2, $3, $4, $5, $6)
RETURNING
  id
