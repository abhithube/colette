SELECT
  id
FROM
  profile_feeds
WHERE
  profile_id = ?1
  AND feed_id = ?2;