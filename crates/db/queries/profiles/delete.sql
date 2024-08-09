WITH
  p AS (
    SELECT
      id,
      is_default
    FROM
      profile
    WHERE
      id = $1
      AND user_id = $2
  ),
  p_delete AS (
    DELETE FROM profile USING p
    WHERE
      profile.id = p.id
      AND NOT p.is_default
  )
SELECT
  p.is_default
FROM
  p;