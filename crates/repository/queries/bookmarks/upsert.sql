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
ON CONFLICT (user_id, link) DO UPDATE
SET
  title = EXCLUDED.title,
  thumbnail_url = EXCLUDED.thumbnail_url,
  published_at = EXCLUDED.published_at,
  author = EXCLUDED.author
RETURNING
  id
