UPDATE profiles
SET
  title = coalesce(?3, title),
  image_url = coalesce(?4, image_url)
WHERE
  id = ?1
  AND user_id = ?2 RETURNING id,
  title,
  image_url,
  user_id,
  created_at "created_at: chrono::DateTime<chrono::Utc>",
  updated_at "updated_at: chrono::DateTime<chrono::Utc>";