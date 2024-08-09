SELECT
  t.id,
  t.title,
  count(pb.id) AS bookmark_count,
  count(pf.id) AS feed_count
FROM
  profile_feed_tag AS pft
  INNER JOIN tag AS t ON t.id = pft.tag_id
  LEFT JOIN profile_bookmark_tag AS pbt ON pbt.tag_id = t.id
  LEFT JOIN profile_bookmark AS pb ON pb.id = pbt.profile_bookmark_id
  LEFT JOIN profile_feed AS pf ON pf.id = pft.profile_feed_id
WHERE
  pft.profile_id = $1
GROUP BY
  t.id
ORDER BY
  t.title ASC;