UPDATE accounts
SET
  password_hash = CASE
    WHEN $2 THEN $3
    ELSE accounts.password_hash
  END
WHERE
  id = $1
