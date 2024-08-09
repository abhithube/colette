SELECT
  id,
  coalesce(url, link) AS "url!"
FROM
  feed;