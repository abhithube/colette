UPDATE feeds
SET
  status = 'failed',
  last_refreshed_at = now()
WHERE
  source_url = $1
