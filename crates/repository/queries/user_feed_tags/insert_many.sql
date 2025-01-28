INSERT INTO user_feed_tags (user_feed_id, tag_id, user_id)
SELECT $1 AS user_feed_id, id, user_id
FROM tags
WHERE user_id = $3 AND title = any($2) ON CONFLICT (
    user_feed_id, tag_id
) DO NOTHING
