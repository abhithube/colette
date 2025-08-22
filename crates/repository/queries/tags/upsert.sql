INSERT INTO
  tags (id, title, user_id, created_at, updated_at)
VALUES
  ($1, $2, $3, $4, $5)
ON CONFLICT (id) DO UPDATE
SET
  title = EXCLUDED.title,
  updated_at = EXCLUDED.updated_at
