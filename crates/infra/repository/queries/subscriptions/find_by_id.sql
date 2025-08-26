SELECT
  s.id,
  s.title,
  s.description,
  s.feed_id,
  array_agg(
    st.tag_id
    ORDER BY
      st.created_at ASC
  ) AS "tags!",
  s.user_id,
  s.created_at,
  s.updated_at
FROM
  subscriptions s
  LEFT JOIN subscription_tags st ON st.subscription_id = s.id
WHERE
  s.id = $1
  AND s.user_id = $2
GROUP BY
  s.id
