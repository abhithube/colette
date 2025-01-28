INSERT INTO user_feed_entries (feed_entry_id, user_feed_id, user_id)
SELECT feed_entry_id, uf.id, uf.user_id
FROM unnest($1::uuid []) AS feed_entry_id
JOIN user_feeds uf ON uf.feed_id = $2 ON CONFLICT (
    user_feed_id, feed_entry_id
) DO NOTHING
