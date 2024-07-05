DELETE FROM profile_feeds
WHERE
  id = $1
  AND profile_id = $2;
