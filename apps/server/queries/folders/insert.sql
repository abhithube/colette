INSERT INTO
  folders (title, folder_type, parent_id, user_id)
VALUES
  ($1, $2, $3, $4)
RETURNING
  id
