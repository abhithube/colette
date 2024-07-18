use colette_core::bookmarks::BookmarkExtractorOptions;

pub fn base_extractor_options() -> BookmarkExtractorOptions {
    BookmarkExtractorOptions {
        title_expr: &[
            "/html/head/*[name='title']/@content",
            "/html/head/title/text()",
        ],
        published_expr: &[],
        author_expr: &[],
        thumbnail_expr: &[],
    }
}

pub fn open_graph_extractor_options() -> BookmarkExtractorOptions {
    BookmarkExtractorOptions {
        title_expr: &["/html/head/og:title/@content"],
        published_expr: &[],
        author_expr: &[],
        thumbnail_expr: &["/html/head/og:image/@content"],
    }
}

pub fn twitter_extractor_options() -> BookmarkExtractorOptions {
    BookmarkExtractorOptions {
        title_expr: &["/html/head/twitter:title/@content"],
        published_expr: &[],
        author_expr: &[],
        thumbnail_expr: &["/html/head/twitter:image/@content"],
    }
}

pub fn microdata_extractor_options() -> BookmarkExtractorOptions {
    BookmarkExtractorOptions {
        title_expr: &["/html/head/*[@itemscope and @itemtype='http://schema.org/VideoObject']/*[@itemprop='name']/@content"],
        published_expr: &[
            "html/head/*[@itemscope and @itemtype='http://schema.org/VideoObject']/*[@itemprop='datePublished']/@content",
            "/*[@itemscope and @itemtype='http://schema.org/VideoObject']/*[@itemprop='uploadDate']/@content"
        ],
        author_expr: &["/html/head/*[@itemscope and @itemtype='http://schema.org/Person']/*[@itemprop='name']/@content"],
        thumbnail_expr: &["/html/head/*[@itemscope and @itemtype='http://schema.org/ImageObject']/*[@itemprop='url']/@href", "/*[@itemprop='thumbnailUrl']/@href"],
    }
}
