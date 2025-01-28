INSERT INTO user_feeds (title, folder_id, feed_id, user_id) VALUES (
    $1, $2, $3, $4
) RETURNING id
