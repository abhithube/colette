INSERT INTO tags (title, user_id)
SELECT *, $2 AS user_id
FROM unnest($1::text []) ON CONFLICT (user_id, title) DO NOTHING
