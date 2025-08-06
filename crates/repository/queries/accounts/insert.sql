INSERT INTO
  accounts (sub, provider, password_hash, user_id)
VALUES
  ($1, $2, $3, $4)
RETURNING
  id
