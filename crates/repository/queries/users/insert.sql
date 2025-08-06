INSERT INTO
  users (email, display_name, image_url)
VALUES
  ($1, $2, $3)
RETURNING
  id
