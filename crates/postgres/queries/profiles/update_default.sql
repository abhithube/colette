WITH
  updated AS (
    UPDATE profiles
    SET
      is_default = FALSE
    WHERE
      user_id = $2
      AND is_default = TRUE
  )
UPDATE profiles
SET
  is_default = TRUE
WHERE
  id = $1
  AND user_id = $2
RETURNING
  id,
  title,
  image_url,
  user_id,
  created_at,
  updated_at;