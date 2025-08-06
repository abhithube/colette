UPDATE users
SET
  display_name = CASE
    WHEN $2 THEN $3
    ELSE users.display_name
  END,
  image_url = CASE
    WHEN $4 THEN $5
    ELSE users.image_url
  END
WHERE
  id = $1
