use colette_core::feeds::ExtractorOptions;

pub struct AtomExtractorOptions(ExtractorOptions);

impl AtomExtractorOptions {
    pub fn inner(self) -> ExtractorOptions {
        self.0
    }
}

impl Default for AtomExtractorOptions {
    fn default() -> Self {
        Self(ExtractorOptions {
            feed_link_expr: Some("//feed/link[@rel='alternate']/@href"),
            feed_title_expr: "//feed/title/text()",
            feed_entries_expr: "//feed/entry",
            entry_link_expr: "./link/@href",
            entry_title_expr: "./title/text()",
            entry_published_expr: Some("./published/text()"),
            entry_description_expr: Some("./content/text()"),
            entry_author_expr: Some("./author/name/text()"),
            entry_thumbnail_expr: None,
        })
    }
}

pub struct RssExtractorOptions(ExtractorOptions);

impl RssExtractorOptions {
    pub fn inner(self) -> ExtractorOptions {
        self.0
    }
}

impl Default for RssExtractorOptions {
    fn default() -> Self {
        Self(ExtractorOptions {
            feed_link_expr: Some("/rss/channel/link/text()"),
            feed_title_expr: "/rss/channel/title/text()",
            feed_entries_expr: "/rss/channel/item",
            entry_link_expr: "/link/text()",
            entry_title_expr: "/title/text()",
            entry_published_expr: Some("/pubDate/text()"),
            entry_description_expr: Some("/description/text()"),
            entry_author_expr: Some("/author/text()"),
            entry_thumbnail_expr: Some("/enclosure[starts-with(@type, 'image/')]/@url"),
        })
    }
}
