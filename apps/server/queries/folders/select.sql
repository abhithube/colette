WITH RECURSIVE
  folder_tree AS (
    SELECT
      id,
      title,
      parent_id,
      created_at,
      updated_at,
      '[]'::jsonb AS path
    FROM
      folders
    WHERE
      user_id = $1
      AND parent_id IS NULL
    UNION ALL
    SELECT
      f.id,
      f.title,
      f.parent_id,
      f.created_at,
      f.updated_at,
      ft.path || jsonb_build_object('id', ft.id, 'title', ft.title)
    FROM
      folders f
      INNER JOIN folder_tree ft ON ft.id = f.parent_id
  )
SELECT
  id AS "id!",
  title AS "title!",
  parent_id,
  created_at,
  updated_at,
  path AS "path!: Json<Vec<FolderPathItem>>"
FROM
  folder_tree
WHERE
  (
    $2::BOOLEAN
    OR id = $3
  )
  AND (
    $4::BOOLEAN
    OR CASE
      WHEN $5::uuid IS NULL THEN parent_id IS NULL
      ELSE parent_id = $5
    END
  )
  AND (
    $6::BOOLEAN
    OR title > $7
  )
ORDER BY
  title ASC
LIMIT
  $8
