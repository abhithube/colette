SELECT
  id,
  email,
  password,
  created_at "created_at: chrono::DateTime<chrono::Utc>",
  updated_at "updated_at: chrono::DateTime<chrono::Utc>"
FROM
  users
WHERE
  email = ?1;