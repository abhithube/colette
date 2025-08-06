UPDATE jobs
SET
  status = CASE
    WHEN $2::TEXT IS NOT NULL THEN $2
    ELSE jobs.status
  END,
  message = CASE
    WHEN $3 THEN $4
    ELSE jobs.message
  END
WHERE
  id = $1
