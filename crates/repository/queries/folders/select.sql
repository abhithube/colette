SELECT id, title, parent_id
FROM folders
WHERE
    user_id = $1
    AND ($2::bool OR id = $3)
    AND (
        $4::bool
        OR CASE
            WHEN $5::uuid IS NULL THEN parent_id IS NULL ELSE parent_id = $5
        END
    )
    AND ($6::bool OR title > $7)
ORDER BY title ASC
LIMIT $8
