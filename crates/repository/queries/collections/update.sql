UPDATE collections
SET
  title = CASE
    WHEN $2::TEXT IS NOT NULL THEN $2
    ELSE collections.title
  END,
  filter_json = CASE
    WHEN $3::JSONB IS NOT NULL THEN $3
    ELSE collections.filter_json
  END
WHERE
  id = $1
