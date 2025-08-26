WITH
  input_bookmarks AS (
    SELECT
      *,
      $1::UUID AS user_id
    FROM
      unnest(
        $2::TEXT[],
        $3::TEXT[],
        $4::TEXT[],
        $5::TIMESTAMPTZ[],
        $6::TEXT[],
        $7::TIMESTAMPTZ[],
        $8::TIMESTAMPTZ[]
      ) AS t (
        link,
        title,
        thumbnail_url,
        published_at,
        author,
        created_at,
        updated_at
      )
  ),
  input_tags AS (
    SELECT
      *,
      $1::UUID AS user_id
    FROM
      unnest($9::TEXT[]) AS t (title)
  ),
  input_relationships AS (
    SELECT
      *
    FROM
      unnest($10::TEXT[], $11::TEXT[]) AS t (bookmark_link, tag_title)
  ),
  upserted_bookmarks AS (
    INSERT INTO
      bookmarks (
        link,
        title,
        thumbnail_url,
        published_at,
        author,
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
      created_at = EXCLUDED.created_at,
      updated_at = EXCLUDED.updated_at
    RETURNING
      id,
      link
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
  bookmark_tags (bookmark_id, tag_id)
SELECT
  b.id,
  t.id
FROM
  input_relationships i
  JOIN upserted_bookmarks b ON b.link = i.bookmark_link
  JOIN new_tags t ON t.title = i.tag_title
ON CONFLICT (bookmark_id, tag_id) DO NOTHING
