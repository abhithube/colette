INSERT INTO
  streams (title, "filter", user_id)
VALUES
  ($1, $2, $3)
RETURNING
  id
