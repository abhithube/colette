INSERT INTO
  feeds (link, xml_url, updated_at)
VALUES
  ($1, $2, now())
ON CONFLICT (link) DO UPDATE
SET
  xml_url = excluded.xml_url,
  updated_at = excluded.updated_at
RETURNING
  id
