WITH
  u AS (
    INSERT INTO
      users (email, password)
    VALUES
      ($1, $2)
    RETURNING
      id,
      email,
      password
  ),
  p AS (
    INSERT INTO
      profiles (title, is_default, user_id)
    SELECT
      'Default',
      TRUE,
      u.id
    FROM
      u
  )
SELECT
  u.id,
  u.email,
  u.password
FROM
  u;