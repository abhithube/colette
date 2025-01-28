UPDATE user_bookmarks SET
    title = CASE WHEN $3 THEN $4 ELSE title END,
    thumbnail_url = CASE WHEN $5 THEN $6 ELSE thumbnail_url END,
    published_at = CASE WHEN $7 THEN $8 ELSE published_at END,
    author = CASE WHEN $9 THEN $10 ELSE author END,
    folder_id = CASE WHEN $11 THEN $12 ELSE folder_id END,
    updated_at = now()
WHERE id = $1 AND user_id = $2
