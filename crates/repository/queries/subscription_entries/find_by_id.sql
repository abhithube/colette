SELECT
  se.id,
  s.user_id
FROM
  subscription_entries se
  INNER JOIN subscriptions s ON s.id = se.subscription_id
WHERE
  se.id = $1
