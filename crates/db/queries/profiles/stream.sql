SELECT
  p.id
FROM
  profile AS p
  INNER JOIN profile_feed AS pf ON pf.profile_id = p.id
WHERE
  pf.feed_id = $1;