WITH
  new_feed AS (
    INSERT INTO
      feeds (link, title, xml_url, updated_at)
    VALUES
      ($1, $2, $3, now())
    ON CONFLICT (link) DO UPDATE
    SET
      title = excluded.title,
      xml_url = excluded.xml_url,
      updated_at = excluded.updated_at
    RETURNING
      id
  ),
  new_feed_entries AS (
    INSERT INTO
      feed_entries (
        link,
        title,
        published_at,
        description,
        author,
        thumbnail_url,
        feed_id,
        updated_at
      )
    SELECT
      *,
      now()
    FROM
      unnest(
        $4::TEXT[],
        $5::TEXT[],
        $6::TIMESTAMPTZ[],
        $7::TEXT[],
        $8::TEXT[],
        $9::TEXT[]
      ),
      new_feed
    ON CONFLICT (feed_id, link) DO UPDATE
    SET
      title = excluded.title,
      published_at = excluded.published_at,
      description = excluded.description,
      author = excluded.author,
      thumbnail_url = excluded.thumbnail_url,
      updated_at = excluded.updated_at
  )
SELECT
  id
FROM
  new_feed
