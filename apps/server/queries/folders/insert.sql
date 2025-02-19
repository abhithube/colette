INSERT INTO
  folders (title, parent_id, user_id)
VALUES
  ($1, $2, $3)
RETURNING
  id
