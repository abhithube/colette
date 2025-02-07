UPDATE folders
SET
  title = CASE
    WHEN $3 THEN $4
    ELSE title
  END,
  parent_id = CASE
    WHEN $5 THEN $6
    ELSE parent_id
  END
WHERE
  id = $1
  AND user_id = $2
