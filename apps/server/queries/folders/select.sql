SELECT
  id,
  title,
  folder_type AS "folder_type: FolderType",
  parent_id,
  created_at,
  updated_at
FROM
  folders
WHERE
  user_id = $1
  AND (
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
