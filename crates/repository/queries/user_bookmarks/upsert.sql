INSERT INTO user_bookmarks (
    title, thumbnail_url, published_at, author, folder_id, bookmark_id, user_id
) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (
    user_id, bookmark_id
) DO NOTHING
