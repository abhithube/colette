WITH
  input_feeds AS (
    SELECT
      *,
      $5::INT AS refresh_interval_min
    FROM
      unnest($2::TEXT[], $3::TEXT[], $4::TEXT[]) AS t (source_url, link, title)
  ),
  input_subscriptions AS (
    SELECT
      *,
      $1::UUID AS user_id
    FROM
      unnest($2::TEXT[], $4::TEXT[]) AS t (source_url, title)
  ),
  input_tags AS (
    SELECT
      *,
      $1::UUID AS user_id
    FROM
      unnest($6::TEXT[]) AS t (title)
  ),
  input_relationships AS (
    SELECT
      *
    FROM
      unnest($7::TEXT[], $8::TEXT[]) AS t (feed_source_url, tag_title)
  ),
  f AS (
    INSERT INTO
      feeds (source_url, link, title, refresh_interval_min)
    SELECT
      source_url,
      link,
      title,
      refresh_interval_min
    FROM
      input_feeds
    ON CONFLICT (source_url) DO UPDATE
    SET
      link = EXCLUDED.link,
      title = EXCLUDED.title
    RETURNING
      id,
      source_url
  ),
  s AS (
    INSERT INTO
      subscriptions (title, feed_id, user_id)
    SELECT
      s.title,
      f.id,
      s.user_id
    FROM
      input_subscriptions s
      JOIN f ON f.source_url = s.source_url
    ON CONFLICT (user_id, feed_id) DO UPDATE
    SET
      title = EXCLUDED.title
    RETURNING
      id,
      feed_id
  ),
  fe AS (
    SELECT
      fe.id,
      fe.feed_id
    FROM
      feed_entries fe
      JOIN f ON f.id = fe.feed_id
  ),
  upserted_tags AS (
    INSERT INTO
      tags (title, user_id)
    SELECT
      title,
      user_id
    FROM
      input_tags
    ON CONFLICT (user_id, title) DO NOTHING
  ),
  new_tags AS (
    SELECT
      id,
      title
    FROM
      tags
    WHERE
      title = ANY (
        SELECT
          title
        FROM
          input_tags
      )
  )
INSERT INTO
  subscription_tags (subscription_id, tag_id)
SELECT
  s.id,
  t.id
FROM
  input_relationships i
  JOIN f ON f.source_url = i.feed_source_url
  JOIN s ON s.feed_id = f.id
  JOIN new_tags t ON t.title = i.tag_title
ON CONFLICT (subscription_id, tag_id) DO NOTHING
