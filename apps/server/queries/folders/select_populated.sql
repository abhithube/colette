WITH RECURSIVE
  folder_tree AS (
    SELECT DISTINCT
      f.id,
      f.title,
      f.parent_id
    FROM
      folders f
      LEFT JOIN user_feeds uf ON uf.folder_id = f.id
      LEFT JOIN bookmarks b ON b.folder_id = f.id
    WHERE
      f.user_id = $1
      AND (
        (
          $2
          AND uf.id IS NOT NULL
        )
        OR (
          $3
          AND b.id IS NOT NULL
        )
      )
    UNION
    SELECT
      f.id,
      f.title,
      f.parent_id
    FROM
      folders f
      INNER JOIN folder_tree ft ON f.id = ft.parent_id
    WHERE
      f.user_id = $1
  )
SELECT DISTINCT
  id AS "id!",
  title AS "title!",
  parent_id
FROM
  folder_tree;
