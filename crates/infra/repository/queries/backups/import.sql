WITH
  f AS (
    SELECT
      id,
      source_url
    FROM
      feeds
    WHERE
      source_url = ANY ($2::TEXT[])
  ),
  input_subscriptions AS (
    SELECT
      *,
      $1::UUID AS user_id
    FROM
      unnest(
        $2::TEXT[],
        $3::TEXT[],
        $4::TEXT[],
        $5::TIMESTAMPTZ[],
        $6::TIMESTAMPTZ[]
      ) AS t (
        source_url,
        title,
        description,
        created_at,
        updated_at
      )
  ),
  input_bookmarks AS (
    SELECT
      *,
      $1::UUID AS user_id
    FROM
      unnest(
        $7::TEXT[],
        $8::TEXT[],
        $9::TEXT[],
        $10::TIMESTAMPTZ[],
        $11::TEXT[],
        $12::TEXT[],
        $13::TIMESTAMPTZ[],
        $14::TIMESTAMPTZ[]
      ) AS t (
        link,
        title,
        thumbnail_url,
        published_at,
        author,
        archived_path,
        created_at,
        updated_at
      )
  ),
  input_tags AS (
    SELECT
      *,
      $1::UUID AS user_id
    FROM
      unnest(
        $15::TEXT[],
        $16::TIMESTAMPTZ[],
        $17::TIMESTAMPTZ[]
      ) AS t (title, created_at, updated_at)
  ),
  input_st_relationships AS (
    SELECT
      *
    FROM
      unnest($18::TEXT[], $19::TEXT[]) AS t (feed_source_url, tag_title)
  ),
  input_bt_relationships AS (
    SELECT
      *
    FROM
      unnest($20::TEXT[], $21::TEXT[]) AS t (bookmark_link, tag_title)
  ),
  s AS (
    INSERT INTO
      subscriptions (title, feed_id, user_id)
    SELECT
      s.title,
      f.id,
      s.user_id
    FROM
      f
      JOIN input_subscriptions s ON s.source_url = f.source_url
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
  orphaned_subscriptions AS (
    DELETE FROM subscriptions old USING s
    WHERE
      old.user_id = $1
      AND NOT old.feed_id = s.feed_id
  ),
  b AS (
    INSERT INTO
      bookmarks (
        link,
        title,
        thumbnail_url,
        published_at,
        author,
        archived_path,
        user_id,
        created_at,
        updated_at
      )
    SELECT
      link,
      title,
      thumbnail_url,
      published_at,
      author,
      archived_path,
      user_id,
      created_at,
      updated_at
    FROM
      input_bookmarks
    ON CONFLICT (user_id, link) DO UPDATE
    SET
      title = EXCLUDED.title,
      thumbnail_url = EXCLUDED.thumbnail_url,
      published_at = EXCLUDED.published_at,
      author = EXCLUDED.author,
      archived_path = EXCLUDED.archived_path,
      created_at = EXCLUDED.created_at,
      updated_at = EXCLUDED.updated_at
    RETURNING
      id,
      link
  ),
  orphaned_bookmarks AS (
    DELETE FROM bookmarks b USING input_bookmarks i
    WHERE
      b.user_id = $1
      AND NOT b.link = i.link
  ),
  t AS (
    INSERT INTO
      tags (title, user_id, created_at, updated_at)
    SELECT
      title,
      user_id,
      created_at,
      updated_at
    FROM
      input_tags
    ON CONFLICT (user_id, title) DO UPDATE
    SET
      created_at = EXCLUDED.created_at,
      updated_at = EXCLUDED.updated_at
    RETURNING
      id,
      title
  ),
  orphaned_tags AS (
    DELETE FROM tags t USING input_tags i
    WHERE
      t.user_id = $1
      AND NOT t.title = i.title
  ),
  st AS (
    INSERT INTO
      subscription_tags (subscription_id, tag_id)
    SELECT
      s.id,
      t.id
    FROM
      input_st_relationships i
      JOIN f ON f.source_url = i.feed_source_url
      JOIN s ON s.feed_id = f.id
      JOIN t ON t.title = i.tag_title
    ON CONFLICT (subscription_id, tag_id) DO NOTHING
  )
INSERT INTO
  bookmark_tags (bookmark_id, tag_id)
SELECT
  b.id,
  t.id
FROM
  input_bt_relationships i
  JOIN b ON b.link = i.bookmark_link
  JOIN t ON t.title = i.tag_title
ON CONFLICT (bookmark_id, tag_id) DO NOTHING
