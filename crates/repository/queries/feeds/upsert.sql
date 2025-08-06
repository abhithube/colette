WITH
  upserted_feed AS (
    INSERT INTO
      feeds (
        source_url,
        link,
        title,
        description,
        refresh_interval_min,
        status,
        refreshed_at,
        is_custom
      )
    VALUES
      ($1, $2, $3, $4, $5, 'healthy', now(), $6)
    ON CONFLICT (source_url) DO UPDATE
    SET
      link = EXCLUDED.link,
      title = EXCLUDED.title,
      description = EXCLUDED.description,
      refresh_interval_min = EXCLUDED.refresh_interval_min,
      status = EXCLUDED.status,
      refreshed_at = EXCLUDED.refreshed_at,
      is_custom = EXCLUDED.is_custom
    RETURNING
      id
  ),
  input_fes AS (
    SELECT
      *
    FROM
      unnest(
        $7::TEXT[],
        $8::TEXT[],
        $9::TIMESTAMPTZ[],
        $10::TEXT[],
        $11::TEXT[],
        $12::TEXT[]
      ) AS t (
        link,
        title,
        published_at,
        description,
        author,
        thumbnail_url
      )
  ),
  upserted_fes AS (
    INSERT INTO
      feed_entries (
        link,
        title,
        published_at,
        description,
        author,
        thumbnail_url,
        feed_id
      )
    SELECT
      fe.link,
      fe.title,
      fe.published_at,
      fe.description,
      fe.author,
      fe.thumbnail_url,
      f.id
    FROM
      input_fes fe
      JOIN upserted_feed f ON TRUE
    ON CONFLICT (feed_id, link) DO UPDATE
    SET
      title = EXCLUDED.title,
      published_at = EXCLUDED.published_at,
      description = EXCLUDED.description,
      author = EXCLUDED.author,
      thumbnail_url = EXCLUDED.thumbnail_url
    RETURNING
      id,
      feed_id
  ),
  s AS (
    SELECT
      s.id,
      s.feed_id
    FROM
      subscriptions s
      JOIN upserted_feed f ON f.id = s.feed_id
  ),
  upserted_ses AS (
    INSERT INTO
      subscription_entries (subscription_id, feed_entry_id)
    SELECT
      s.id,
      fe.id
    FROM
      upserted_fes fe
      JOIN s ON s.feed_id = fe.feed_id
    ON CONFLICT DO NOTHING
  )
SELECT
  id
FROM
  upserted_feed
