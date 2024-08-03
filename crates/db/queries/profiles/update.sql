UPDATE profiles AS p
SET
  title = coalesce($3, p.title),
  image_url = coalesce($4, p.image_url)
WHERE
  id = $1
  AND user_id = $2
RETURNING
  id,
  title,
  image_url,
  is_default,
  user_id;