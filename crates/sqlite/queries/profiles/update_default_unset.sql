UPDATE profiles
SET
  is_default = false
WHERE
  user_id = ?1
  AND is_default = true RETURNING id;