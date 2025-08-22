INSERT INTO
  collections (
    id,
    title,
    filter_json,
    user_id,
    created_at,
    updated_at
  )
VALUES
  ($1, $2, $3, $4, $5, $6)
ON CONFLICT (id) DO UPDATE
SET
  title = EXCLUDED.title,
  filter_json = EXCLUDED.filter_json,
  updated_at = EXCLUDED.updated_at
