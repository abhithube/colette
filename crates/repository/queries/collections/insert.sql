INSERT INTO
  collections (title, filter_json, user_id)
VALUES
  ($1, $2, $3)
RETURNING
  id
