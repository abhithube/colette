SELECT
  id,
  title,
  filter_json AS "filter_json: Json<BookmarkFilter>",
  user_id,
  created_at,
  updated_at
FROM
  collections
WHERE
  id = $1
  AND user_id = $2
