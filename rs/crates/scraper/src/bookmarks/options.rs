use colette_core::bookmarks::BookmarkExtractorOptions;

pub fn base_extractor_options<'a>() -> BookmarkExtractorOptions<'a> {
    BookmarkExtractorOptions {
        title_expr: vec![
            "/html/head/*[@name='title']/@content",
            "/html/head/title/text()",
        ],
        published_expr: vec![],
        author_expr: vec![],
        thumbnail_expr: vec![],
    }
}

pub fn open_graph_extractor_options<'a>() -> BookmarkExtractorOptions<'a> {
    BookmarkExtractorOptions {
        title_expr: vec!["//*[@property='og:title']/@content"],
        published_expr: vec![],
        author_expr: vec![],
        thumbnail_expr: vec!["//*[@property='og:image']/@content"],
    }
}

pub fn twitter_extractor_options<'a>() -> BookmarkExtractorOptions<'a> {
    BookmarkExtractorOptions {
        title_expr: vec!["//*[@name='twitter:title']/@content"],
        published_expr: vec![],
        author_expr: vec![],
        thumbnail_expr: vec!["//*[@name='twitter:image']/@content"],
    }
}

pub fn microdata_extractor_options<'a>() -> BookmarkExtractorOptions<'a> {
    BookmarkExtractorOptions {
        title_expr: vec![
            "//*[@itemtype='http://schema.org/VideoObject']/*[@itemprop='name']/@content",
        ],
        published_expr: vec![
            "//*[@itemtype='http://schema.org/VideoObject']/*[@itemprop='datePublished']/@content",
            "//*[@itemtype='http://schema.org/VideoObject']/*[@itemprop='uploadDate']/@content",
        ],
        author_expr: vec!["//*[@itemtype='http://schema.org/Person']/*[@itemprop='name']/@content"],
        thumbnail_expr: vec![
            "//*[@itemtype='http://schema.org/ImageObject']/*[@itemprop='url']/@href",
            "//*[@itemprop='thumbnailUrl']/@href",
        ],
    }
}
