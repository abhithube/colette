use colette_core::feeds::FeedExtractorOptions;

#[derive(Clone, Debug)]
pub struct AtomExtractorOptions(FeedExtractorOptions);

impl AtomExtractorOptions {
    pub fn inner(self) -> FeedExtractorOptions {
        self.0
    }
}

impl Default for AtomExtractorOptions {
    fn default() -> Self {
        Self(FeedExtractorOptions {
            feed_link_expr: &["/atom:feed/atom:link[@rel='alternate']/@href"],
            feed_title_expr: &["/atom:feed/atom:title/text()"],
            feed_entries_expr: &["/atom:feed/atom:entry"],
            entry_link_expr: &["./atom:link/@href"],
            entry_title_expr: &["./atom:title/text()"],
            entry_published_expr: &["./atom:published/text()"],
            entry_description_expr: &["./atom:content/text()"],
            entry_author_expr: &["./atom:author/atom:name/text()"],
            entry_thumbnail_expr: &[],
        })
    }
}

#[derive(Clone, Debug)]
pub struct RssExtractorOptions(FeedExtractorOptions);

impl RssExtractorOptions {
    pub fn inner(self) -> FeedExtractorOptions {
        self.0
    }
}

impl Default for RssExtractorOptions {
    fn default() -> Self {
        Self(FeedExtractorOptions {
            feed_link_expr: &["/rss/channel/link/text()"],
            feed_title_expr: &["/rss/channel/title/text()"],
            feed_entries_expr: &["/rss/channel/item"],
            entry_link_expr: &["./link/text()"],
            entry_title_expr: &["./title/text()"],
            entry_published_expr: &["./pubDate/text()"],
            entry_description_expr: &["./description/text()"],
            entry_author_expr: &["./author/text()"],
            entry_thumbnail_expr: &["./enclosure[starts-with(@type, 'image/')]/@url"],
        })
    }
}

#[derive(Clone, Debug)]
pub struct MediaExtractorOptions(FeedExtractorOptions);

impl MediaExtractorOptions {
    pub fn inner(self) -> FeedExtractorOptions {
        self.0
    }
}

impl Default for MediaExtractorOptions {
    fn default() -> Self {
        Self(FeedExtractorOptions {
            feed_link_expr: &[],
            feed_title_expr: &[],
            feed_entries_expr: &[],
            entry_link_expr: &[],
            entry_title_expr: &["./media:group/media:title/text()", "./media:title/text()"],
            entry_published_expr: &[],
            entry_description_expr: &[
                "./media:group/media:description/text()",
                "./media:description/text()",
            ],
            entry_author_expr: &[],
            entry_thumbnail_expr: &[
                "./media:group/media:thumbnail/@url",
                "./media:thumbnail/@url",
            ],
        })
    }
}

#[derive(Clone, Debug)]
pub struct DublinCoreExtractorOptions(FeedExtractorOptions);

impl DublinCoreExtractorOptions {
    pub fn inner(self) -> FeedExtractorOptions {
        self.0
    }
}

impl Default for DublinCoreExtractorOptions {
    fn default() -> Self {
        Self(FeedExtractorOptions {
            feed_link_expr: &[],
            feed_title_expr: &["/rss/channel/dc:title/text()"],
            feed_entries_expr: &[],
            entry_link_expr: &[],
            entry_title_expr: &["./dc:title/text()"],
            entry_published_expr: &["./dc:date/text()"],
            entry_description_expr: &["./dc:description/text()"],
            entry_author_expr: &["./dc:creator/text()"],
            entry_thumbnail_expr: &[],
        })
    }
}

#[derive(Clone, Debug)]
pub struct ITunesExtractorOptions(FeedExtractorOptions);

impl ITunesExtractorOptions {
    pub fn inner(self) -> FeedExtractorOptions {
        self.0
    }
}

impl Default for ITunesExtractorOptions {
    fn default() -> Self {
        Self(FeedExtractorOptions {
            feed_link_expr: &[],
            feed_title_expr: &["/rss/channel/itunes:title/text()"],
            feed_entries_expr: &[],
            entry_link_expr: &[],
            entry_title_expr: &["./itunes:title/text()"],
            entry_published_expr: &[],
            entry_description_expr: &[],
            entry_author_expr: &["/rss/channel/itunes:author/text()"],
            entry_thumbnail_expr: &["./itunes:image/@href"],
        })
    }
}
