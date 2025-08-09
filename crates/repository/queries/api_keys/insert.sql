INSERT INTO
  api_keys (
    lookup_hash,
    verification_hash,
    title,
    preview,
    user_id
  )
VALUES
  ($1, $2, $3, $4, $5)
RETURNING
  id,
  title,
  preview,
  user_id,
  created_at,
  updated_at
