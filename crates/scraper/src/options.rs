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
            feed_link_expr: &["//feed/link[@rel='alternate']/@href"],
            feed_title_expr: &["//feed/title/text()"],
            feed_entries_expr: &["//feed/entry"],
            entry_link_expr: &["./link/@href"],
            entry_title_expr: &[
                "./media:group/media:title/text()",
                "./media:title/text()",
                "./title/text()",
            ],
            entry_published_expr: &["./published/text()"],
            entry_description_expr: &[
                "./media:group/media:description/text()",
                "./media:description/text()",
                "./content/text()",
            ],
            entry_author_expr: &["./author/name/text()"],
            entry_thumbnail_expr: &[
                "./media:group/media:thumbnail/@url",
                "./media:thumbnail/@url",
            ],
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
            feed_link_expr: &["/rss/channel/link/text()"],
            feed_title_expr: &["/rss/channel/title/text()"],
            feed_entries_expr: &["/rss/channel/item"],
            entry_link_expr: &["/link/text()"],
            entry_title_expr: &["/title/text()"],
            entry_published_expr: &["/pubDate/text()"],
            entry_description_expr: &["/description/text()"],
            entry_author_expr: &["./dc:creator/text()", "/author/text()"],
            entry_thumbnail_expr: &["/enclosure[starts-with(@type, 'image/')]/@url"],
        })
    }
}
