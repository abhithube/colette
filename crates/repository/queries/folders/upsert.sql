WITH new_folder AS (
    INSERT INTO folders (title, parent_id, user_id) VALUES (
        $1, $2, $3
    ) ON CONFLICT (user_id, parent_id, title) DO NOTHING RETURNING id
)

SELECT id AS "id!" FROM new_folder
UNION ALL
SELECT id FROM folders WHERE user_id = $3 AND parent_id = $2 AND title = $1
