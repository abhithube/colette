DELETE FROM profile_feed
WHERE
  id = $1
  AND profile_id = $2;