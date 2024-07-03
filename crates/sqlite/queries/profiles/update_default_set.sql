UPDATE profiles
SET
  is_default = 1
WHERE
  id = ?1
  AND user_id = ?2 RETURNING id,
  title,
  image_url,
  user_id,
  created_at "created_at: chrono::DateTime<chrono::Utc>",
  updated_at "updated_at: chrono::DateTime<chrono::Utc>";