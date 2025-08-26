WITH
  upserted_feed AS (
    INSERT INTO
      feeds (
        id,
        source_url,
        link,
        title,
        description,
        is_custom,
        status,
        last_refreshed_at,
        created_at,
        updated_at
      )
    VALUES
      ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
    ON CONFLICT (source_url) DO UPDATE
    SET
      link = EXCLUDED.link,
      title = EXCLUDED.title,
      description = EXCLUDED.description,
      status = EXCLUDED.status,
      last_refreshed_at = EXCLUDED.last_refreshed_at,
      is_custom = EXCLUDED.is_custom,
      updated_at = EXCLUDED.updated_at
  ),
  input_fes AS (
    SELECT
      *,
      $1 AS feed_id
    FROM
      unnest(
        $11::UUID[],
        $12::TEXT[],
        $13::TEXT[],
        $14::TIMESTAMPTZ[],
        $15::TEXT[],
        $16::TEXT[],
        $17::UUID[],
        $18::TIMESTAMPTZ[],
        $19::TIMESTAMPTZ[]
      ) AS t (
        id,
        link,
        title,
        published_at,
        description,
        author,
        thumbnail_url,
        created_at,
        updated_at
      )
  )
INSERT INTO
  feed_entries (
    id,
    link,
    title,
    published_at,
    description,
    author,
    thumbnail_url,
    feed_id,
    created_at,
    updated_at
  )
SELECT
  fe.id,
  fe.link,
  fe.title,
  fe.published_at,
  fe.description,
  fe.author,
  fe.thumbnail_url,
  fe.feed_id,
  fe.created_at,
  fe.updated_at
FROM
  input_fes fe
ON CONFLICT (feed_id, link) DO UPDATE
SET
  title = EXCLUDED.title,
  published_at = EXCLUDED.published_at,
  description = EXCLUDED.description,
  author = EXCLUDED.author,
  thumbnail_url = EXCLUDED.thumbnail_url,
  updated_at = EXCLUDED.updated_at
