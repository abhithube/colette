UPDATE api_keys
SET
  title = CASE
    WHEN $2::TEXT IS NOT NULL THEN $2
    ELSE api_keys.title
  END
WHERE
  id = $1
