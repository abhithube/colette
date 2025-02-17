WITH RECURSIVE
  folder_tree AS (
    SELECT DISTINCT
      f.id,
      f.title,
      f.folder_type,
      f.parent_id,
      f.created_at,
      f.updated_at
    FROM
      folders f
      LEFT JOIN user_feeds uf ON uf.folder_id = f.id
      LEFT JOIN collections c ON c.folder_id = f.id
    WHERE
      f.user_id = $1
      AND (
        (
          $2
          AND uf.id IS NOT NULL
        )
        OR (
          $3
          AND c.id IS NOT NULL
        )
      )
    UNION
    SELECT
      f.id,
      f.title,
      f.folder_type,
      f.parent_id,
      f.created_at,
      f.updated_at
    FROM
      folders f
      INNER JOIN folder_tree ft ON f.id = ft.parent_id
    WHERE
      f.user_id = $1
  )
SELECT DISTINCT
  id AS "id!",
  title AS "title!",
  folder_type AS "folder_type!: FolderType",
  parent_id,
  created_at AS "created_at!",
  updated_at AS "updated_at!"
FROM
  folder_tree;
