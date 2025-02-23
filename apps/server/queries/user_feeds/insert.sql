INSERT INTO
  user_feeds (title, feed_id, user_id)
VALUES
  ($1, $2, $3)
RETURNING
  id Z
