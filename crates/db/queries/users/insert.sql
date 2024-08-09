WITH
  u AS (
    INSERT INTO
      "user" (email, password)
    VALUES
      ($1, $2)
    RETURNING
      id,
      email,
      password
  ),
  p AS (
    INSERT INTO
      profile (title, is_default, user_id)
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