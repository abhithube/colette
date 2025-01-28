DELETE FROM user_bookmark_tags
WHERE
    user_id = $1
    AND tag_id IN (SELECT id FROM tags WHERE user_id = $1 AND title = any($2))
