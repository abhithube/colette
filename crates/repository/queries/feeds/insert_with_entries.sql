WITH new_feed AS (
    INSERT INTO feeds (link, title, xml_url, updated_at) VALUES (
        $1, $2, $3, now()
    ) ON CONFLICT (link) DO UPDATE SET title
    = excluded.title,
    xml_url = excluded.xml_url,
    updated_at = excluded.updated_at RETURNING id
),

new_feed_entries AS (
    INSERT INTO feed_entries (
        link,
        title,
        published_at,
        description,
        author,
        thumbnail_url,
        feed_id,
        updated_at
    )
    SELECT *, now() AS updated_at
    FROM
        unnest(
            $4::text [],
            $5::text [],
            $6::timestamp [],
            $7::text [],
            $8::text [],
            $9::text []
        ), new_feed
    ON CONFLICT (feed_id, link) DO UPDATE SET title
    = excluded.title,
    published_at = excluded.published_at,
    description = excluded.description,
    author = excluded.author,
    thumbnail_url = excluded.thumbnail_url,
    updated_at = excluded.updated_at
)

SELECT id FROM new_feed
