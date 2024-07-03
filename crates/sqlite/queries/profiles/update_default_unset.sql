UPDATE profiles
SET
  is_default = 0
WHERE
  user_id = ?1
  AND is_default = true RETURNING id;