DELETE FROM subscriptions
WHERE
  id = $1
  AND user_id = $2
