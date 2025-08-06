INSERT INTO
  subscriptions (title, description, feed_id, user_id)
VALUES
  ($1, $2, $3, $4)
RETURNING
  id
