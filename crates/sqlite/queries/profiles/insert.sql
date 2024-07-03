INSERT INTO
  profiles (id, title, image_url, is_default, user_id)
VALUES
  (?1, ?2, ?3, ?4, ?5) RETURNING id,
  title,
  image_url,
  user_id,
  created_at "created_at: chrono::DateTime<chrono::Utc>",
  updated_at "updated_at: chrono::DateTime<chrono::Utc>";
