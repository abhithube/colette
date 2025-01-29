INSERT INTO
  bookmarks (
    link,
    title,
    thumbnail_url,
    published_at,
    author,
    updated_at
  )
VALUES
  ($1, $2, $3, $4, $5, now())
ON CONFLICT (link) DO UPDATE
SET
  title = excluded.title,
  thumbnail_url = excluded.thumbnail_url,
  published_at = excluded.published_at,
  author = excluded.author,
  updated_at = excluded.updated_at
RETURNING
  id
