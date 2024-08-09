WITH
  u AS (
    INSERT INTO
      "user" (id, email, password)
    VALUES
      ($1, $2, $3)
    RETURNING
      id,
      email,
      password
  ),
  p AS (
    INSERT INTO
      profile (id, title, is_default, user_id)
    SELECT
      $4,
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