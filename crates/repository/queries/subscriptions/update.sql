UPDATE subscriptions
SET
  title = CASE
    WHEN $2::TEXT IS NOT NULL THEN $2
    ELSE subscriptions.title
  END,
  description = CASE
    WHEN $3 THEN $4
    ELSE subscriptions.description
  END
WHERE
  id = $1
