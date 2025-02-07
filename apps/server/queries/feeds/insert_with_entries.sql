WITH
  new_feed AS (
    INSERT INTO
      feeds (link, xml_url)
    VALUES
      ($1, $2)
    ON CONFLICT (link) DO UPDATE
    SET
      xml_url = excluded.xml_url
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
        feed_id
      )
    SELECT
      *
    FROM
      unnest(
        $3::TEXT[],
        $4::TEXT[],
        $5::TIMESTAMPTZ[],
        $6::TEXT[],
        $7::TEXT[],
        $8::TEXT[]
      ),
      new_feed
    ON CONFLICT (feed_id, link) DO UPDATE
    SET
      title = excluded.title,
      published_at = excluded.published_at,
      description = excluded.description,
      author = excluded.author,
      thumbnail_url = excluded.thumbnail_url
  )
SELECT
  id
FROM
  new_feed
