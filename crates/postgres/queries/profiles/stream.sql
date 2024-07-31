SELECT
  p.id
FROM
  profiles AS p
  INNER JOIN profile_feeds AS pf ON pf.profile_id = p.id
WHERE
  pf.feed_id = $1;