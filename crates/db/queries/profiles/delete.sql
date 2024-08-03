WITH
  p AS (
    SELECT
      id,
      is_default
    FROM
      profiles
    WHERE
      id = $1
      AND user_id = $2
  ),
  p_delete AS (
    DELETE FROM profiles USING p
    WHERE
      profiles.id = p.id
      AND NOT p.is_default
  )
SELECT
  p.is_default
FROM
  p;