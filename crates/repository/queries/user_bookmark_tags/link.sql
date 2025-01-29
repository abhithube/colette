WITH new_tags AS (
    INSERT INTO tags (title, user_id)
    SELECT
        unnest($3::text []) AS title, $2 AS user_id
        ON CONFLICT (user_id, title) DO NOTHING
        RETURNING id
),

all_tags AS (
    SELECT id FROM new_tags
    UNION ALL
    SELECT id
    FROM tags
    WHERE user_id = $2 AND title = any($3::text [])
),

deleted_ubt AS (
    DELETE FROM user_bookmark_tags ubt
    WHERE
        ubt.user_bookmark_id = $1
        AND ubt.user_id = $2
        AND ubt.tag_id NOT IN (SELECT id FROM all_tags)
)

INSERT INTO user_bookmark_tags (user_bookmark_id, tag_id, user_id)
SELECT $1 AS user_bookmark_id, all_tags.id, $2 AS user_id
FROM all_tags
ON CONFLICT (user_bookmark_id, tag_id) DO NOTHING
