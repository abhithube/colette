SELECT
  coalesce(f.xml_url, f.link) AS "url!"
FROM
  feeds f
  JOIN user_feeds uf ON uf.feed_id = f.id
