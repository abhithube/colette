INSERT INTO
  feeds (link, title, xml_url, updated_at)
VALUES
  ($1, $2, $3, now())
ON CONFLICT (link) DO UPDATE
SET
  title = excluded.title,
  xml_url = excluded.xml_url,
  updated_at = excluded.updated_at
RETURNING
  id
