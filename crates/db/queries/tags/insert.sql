INSERT INTO
  tags (title, profile_id)
VALUES
  ($1, $2)
RETURNING
  id,
  title;