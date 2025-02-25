SELECT
  u.id,
  u.email,
  a.provider_id,
  a.account_id,
  a.password_hash
FROM
  accounts a
  LEFT JOIN users u ON u.id = a.user_id
WHERE
  a.provider_id = $1
  AND a.account_id = $2
