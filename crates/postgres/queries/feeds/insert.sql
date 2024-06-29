INSERT INTO
  feeds (link, title, url)
VALUES
  ($1, $2, $3)
ON CONFLICT (link) DO
UPDATE
SET
  title = excluded.title,
  url = excluded.url
RETURNING
  id;