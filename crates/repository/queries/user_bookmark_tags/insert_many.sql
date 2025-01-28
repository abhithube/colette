INSERT INTO user_bookmark_tags (user_bookmark_id, tag_id, user_id)
SELECT $1 AS user_bookmark_id, id, user_id
FROM tags
WHERE user_id = $3 AND title = any($2) ON CONFLICT (
    user_bookmark_id, tag_id
) DO NOTHING
