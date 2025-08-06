UPDATE subscription_entries
SET
  has_read = TRUE,
  read_at = now()
WHERE
  id = $1
