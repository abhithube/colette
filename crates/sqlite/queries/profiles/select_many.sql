SELECT
  id,
  title,
  image_url,
  user_id,
  created_at "created_at: chrono::DateTime<chrono::Utc>",
  updated_at "updated_at: chrono::DateTime<chrono::Utc>"
FROM
  profiles
WHERE
  user_id = $1;