INSERT INTO
  users (id, email, password)
VALUES
  ($1, $2, $3)
RETURNING
  id,
  email,
  password,
  created_at,
  updated_at;
