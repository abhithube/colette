INSERT INTO
  profile_feeds (profile_id, feed_id)
VALUES
  (?1, ?2)
ON CONFLICT (profile_id, feed_id) DO NOTHING RETURNING id;