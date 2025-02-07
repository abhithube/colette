INSERT INTO
  feeds (link, xml_url)
VALUES
  ($1, $2)
ON CONFLICT (link) DO UPDATE
SET
  xml_url = excluded.xml_url
RETURNING
  id
