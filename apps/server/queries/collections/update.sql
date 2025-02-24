UPDATE collections
SET
  title = CASE
    WHEN $3 THEN $4
    ELSE title
  END,
  "filter" = CASE
    WHEN $5 THEN $6
    ELSE "filter"
  END
WHERE
  id = $1
  AND user_id = $2
