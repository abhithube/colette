INSERT INTO
  entries (
    link,
    title,
    published_at,
    description,
    author,
    thumbnail_url
  )
VALUES
  (?1, ?2, ?3, ?4, ?5, ?6)
ON CONFLICT (link) DO
UPDATE
SET
  title = excluded.title,
  published_at = excluded.published_at,
  description = excluded.description,
  author = excluded.author,
  thumbnail_url = excluded.thumbnail_url RETURNING id;