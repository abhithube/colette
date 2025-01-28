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
SELECT *, $7::uuid AS feed_id, now() AS updated_at
FROM
    unnest(
        $1::text [],
        $2::text [],
        $3::timestamp [],
        $4::text [],
        $5::text [],
        $6::text []
    )
ON CONFLICT (feed_id, link) DO UPDATE SET title
= excluded.title,
published_at = excluded.published_at,
description = excluded.description,
author = excluded.author,
thumbnail_url = excluded.thumbnail_url,
updated_at = excluded.updated_at
