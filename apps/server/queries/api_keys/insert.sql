INSERT INTO
  api_keys (value_hash, value_preview, title, user_id)
VALUES
  ($1, $2, $3, $4)
RETURNING
  id
