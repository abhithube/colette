INSERT INTO
  profile_feeds (id, profile_id, feed_id)
VALUES
  ($1, $2, $3)
ON CONFLICT (profile_id, feed_id) DO nothing
RETURNING
  id;