DELETE FROM personal_access_tokens
WHERE
  id = $1
  AND user_id = $2
