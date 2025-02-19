WITH RECURSIVE
  folder_tree AS (
    SELECT
      id,
      title,
      parent_id,
      created_at,
      updated_at,
      0 AS depth
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
      ft.depth + 1
    FROM
      folders f
      INNER JOIN folder_tree ft ON f.parent_id = ft.id
  ),
  items AS (
    SELECT
      ft.id,
      ft.parent_id,
      ft.title,
      NULL AS href,
      ft.created_at,
      ft.updated_at,
      ft.depth
    FROM
      folder_tree ft
    UNION ALL
    SELECT
      c.id,
      c.folder_id AS parent_id,
      c.title,
      NULL AS href,
      c.created_at,
      c.updated_at,
      coalesce(ft.depth + 1, 0) AS depth
    FROM
      collections c
      LEFT JOIN folder_tree ft ON ft.id = c.folder_id
    WHERE
      c.user_id = $1
  ),
  items_with_bookmarks AS (
    SELECT
      *
    FROM
      items
    UNION ALL
    SELECT
      b.id,
      b.collection_id AS parent_id,
      b.title,
      b.link AS href,
      b.created_at,
      b.updated_at,
      coalesce(i.depth + 1, 0) AS depth
    FROM
      bookmarks b
      LEFT JOIN items i ON i.id = b.collection_id
    WHERE
      b.user_id = $1
  )
SELECT
  id AS "id!",
  parent_id,
  title AS "title!",
  href,
  created_at AS "add_date!",
  updated_at AS "last_modified!"
FROM
  items_with_bookmarks
ORDER BY
  depth ASC
