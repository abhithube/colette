UPDATE user_feeds SET
    title = CASE WHEN $3 THEN $4 ELSE title END,
    folder_id = CASE WHEN $5 THEN $6 ELSE folder_id END,
    updated_at = now()
WHERE id = $1 AND user_id = $2
