SELECT
  id,
  user_id
FROM
  subscriptions
WHERE
  id = $1
