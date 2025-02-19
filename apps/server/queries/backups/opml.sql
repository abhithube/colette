WITH RECURSIVE
  folder_tree AS (
    SELECT
      id,
      title,
      parent_id,
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
      ft.depth + 1
    FROM
      folders f
      INNER JOIN folder_tree ft ON f.parent_id = ft.id
  ),
  outlines AS (
    SELECT
      ft.id,
      ft.parent_id,
      ft.title,
      NULL AS xml_url,
      NULL AS html_url,
      ft.depth
    FROM
      folder_tree ft
    UNION ALL
    SELECT
      uf.id,
      uf.folder_id AS parent_id,
      uf.title,
      f.xml_url,
      f.link AS html_url,
      coalesce(ft.depth + 1, 0) AS depth
    FROM
      user_feeds uf
      LEFT JOIN feeds f ON f.id = uf.feed_id
      LEFT JOIN folder_tree ft ON ft.id = uf.folder_id
    WHERE
      uf.user_id = $1
      AND f.xml_url IS NOT NULL
  )
SELECT
  id AS "id!",
  parent_id,
  title AS "text!",
  xml_url,
  html_url
FROM
  outlines
ORDER BY
  depth ASC
