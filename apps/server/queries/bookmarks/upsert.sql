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
  title = excluded.title,
  thumbnail_url = excluded.thumbnail_url,
  published_at = excluded.published_at,
  author = excluded.author
RETURNING
  id
