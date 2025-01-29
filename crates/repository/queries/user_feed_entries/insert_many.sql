INSERT INTO user_feed_entries (feed_entry_id, user_feed_id, user_id)
SELECT fe.id AS feed_entry_id, uf.id AS user_feed_id, uf.user_id
FROM feed_entries fe
JOIN user_feeds uf ON uf.feed_id = $1
WHERE fe.feed_id = $1
ON CONFLICT (
    user_feed_id, feed_entry_id
) DO NOTHING
