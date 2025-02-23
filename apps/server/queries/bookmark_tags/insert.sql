INSERT INTO
  bookmark_tags (bookmark_id, tag_id, user_id)
VALUES
  ($1, $2, $3)
ON CONFLICT (bookmark_id, tag_id) DO NOTHING
