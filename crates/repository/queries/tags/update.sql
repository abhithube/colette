UPDATE tags SET title = CASE WHEN $3 THEN $4 ELSE title END, updated_at = now()
WHERE id = $1 AND user_id = $2
