UPDATE feeds
SET
  status = 'failed',
  refreshed_at = now()
WHERE
  source_url = $1
