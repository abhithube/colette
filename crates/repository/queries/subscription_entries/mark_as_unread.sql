UPDATE subscription_entries
SET
  has_read = FALSE,
  read_at = NULL
WHERE
  id = $1
