SELECT
  s.id,
  s.title,
  s.description,
  s.feed_id,
  f.source_url AS "source_url: DbUrl",
  f.link AS "link: DbUrl",
  f.title AS feed_title,
  f.description AS feed_description,
  f.refresh_interval_min,
  f.status AS "status: DbFeedStatus",
  f.refreshed_at,
  f.is_custom,
  coalesce(uc.unread_count, 0) AS unread_count,
  coalesce(t.tags, NULL::JSONB) AS "tags: Json<Vec<Tag>>",
  s.user_id,
  s.created_at,
  s.updated_at
FROM
  subscriptions s
  INNER JOIN feeds f ON f.id = s.feed_id
  LEFT JOIN (
    SELECT
      s_inner.id AS subscription_id,
      count(fe.id) AS unread_count
    FROM
      subscriptions s_inner
      INNER JOIN feed_entries fe ON s_inner.feed_id = fe.feed_id
    WHERE
      $7
      AND NOT EXISTS (
        SELECT
          1
        FROM
          subscription_entries se
        WHERE
          se.subscription_id = s_inner.id
          AND se.feed_entry_id = fe.id
      )
    GROUP BY
      s_inner.id
  ) AS uc ON s.id = uc.subscription_id
  LEFT JOIN (
    SELECT
      st.subscription_id,
      jsonb_agg(
        jsonb_build_object(
          'id',
          t.id,
          'title',
          t.title,
          'user_id',
          t.user_id,
          'created_at',
          t.created_at,
          'updated_at',
          t.updated_at
        )
        ORDER BY
          t.title ASC
      ) AS tags
    FROM
      subscription_tags st
      INNER JOIN tags t ON t.id = st.tag_id
    WHERE
      $8
    GROUP BY
      st.subscription_id
  ) AS t ON s.id = t.subscription_id
WHERE
  (
    $1::UUID IS NULL
    OR s.id = $1
  )
  AND (
    $2::UUID IS NULL
    OR s.user_id = $2
  )
  AND (
    $3::UUID[] IS NULL
    OR EXISTS (
      SELECT
        1
      FROM
        subscription_tags st
      WHERE
        st.subscription_id = s.id
        AND st.tag_id = ANY ($3)
    )
  )
  AND (
    (
      $4::TEXT IS NULL
      OR $5::UUID IS NULL
    )
    OR (s.title, s.id) > ($4, $5)
  )
ORDER BY
  s.title ASC,
  s.id ASC
LIMIT
  $6
