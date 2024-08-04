SELECT
  t.id,
  t.title
FROM
  profile_feed_tags pft
  INNER JOIN tags AS t ON t.id = pft.tag_id
WHERE
  pft.profile_id = $1;