SELECT
  EXISTS (
    SELECT
      1
    FROM
      profiles
    WHERE
      user_id = $1
  ) AS "exists!: bool";