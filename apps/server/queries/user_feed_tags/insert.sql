INSERT INTO
  user_feed_tags (user_feed_id, tag_id, user_id)
VALUES
  ($1, $2, $3)
ON CONFLICT (user_feed_id, tag_id) DO NOTHING
