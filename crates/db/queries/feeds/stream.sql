SELECT
  id,
  coalesce(url, link) AS "url!"
FROM
  feeds;