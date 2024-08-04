SELECT
  t.id,
  t.title
FROM
  bookmark_tags bt
  INNER JOIN tags AS t ON t.id = bt.tag_id
WHERE
  bt.profile_id = $1;