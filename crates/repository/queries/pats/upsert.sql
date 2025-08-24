INSERT INTO
  personal_access_tokens (
    id,
    lookup_hash,
    verification_hash,
    title,
    preview,
    user_id,
    created_at,
    updated_at
  )
VALUES
  ($1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT (id) DO UPDATE
SET
  title = EXCLUDED.title,
  updated_at = EXCLUDED.updated_at
