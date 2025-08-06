UPDATE tags
SET
  title = CASE
    WHEN $2::TEXT IS NOT NULL THEN $2
    ELSE tags.title
  END
WHERE
  id = $1
