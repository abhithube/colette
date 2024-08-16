use colette_core::feeds::FeedExtractorOptions;

pub fn atom_extractor_options<'a>() -> FeedExtractorOptions<'a> {
    FeedExtractorOptions {
        feed_link_expr: vec!["/atom:feed/atom:link[@rel='alternate']/@href"],
        feed_title_expr: vec!["/atom:feed/atom:title/text()"],
        feed_entries_expr: vec!["/atom:feed/atom:entry"],
        entry_link_expr: vec!["./atom:link/@href"],
        entry_title_expr: vec!["./atom:title/text()"],
        entry_published_expr: vec!["./atom:published/text()"],
        entry_description_expr: vec!["./atom:content/text()"],
        entry_author_expr: vec!["./atom:author/atom:name/text()"],
        entry_thumbnail_expr: vec![],
    }
}

pub fn rss_extractor_options<'a>() -> FeedExtractorOptions<'a> {
    FeedExtractorOptions {
        feed_link_expr: vec!["/rss/channel/link/text()"],
        feed_title_expr: vec!["/rss/channel/title/text()"],
        feed_entries_expr: vec!["/rss/channel/item"],
        entry_link_expr: vec!["./link/text()"],
        entry_title_expr: vec!["./title/text()"],
        entry_published_expr: vec!["./pubDate/text()"],
        entry_description_expr: vec!["./description/text()"],
        entry_author_expr: vec!["./author/text()"],
        entry_thumbnail_expr: vec!["./enclosure[starts-with(@type, 'image/')]/@url"],
    }
}

pub fn media_extractor_options<'a>() -> FeedExtractorOptions<'a> {
    FeedExtractorOptions {
        feed_link_expr: vec![],
        feed_title_expr: vec![],
        feed_entries_expr: vec![],
        entry_link_expr: vec![],
        entry_title_expr: vec!["./media:group/media:title/text()", "./media:title/text()"],
        entry_published_expr: vec![],
        entry_description_expr: vec![
            "./media:group/media:description/text()",
            "./media:description/text()",
        ],
        entry_author_expr: vec![],
        entry_thumbnail_expr: vec![
            "./media:group/media:thumbnail/@url",
            "./media:thumbnail/@url",
        ],
    }
}

pub fn dublin_core_extractor_options<'a>() -> FeedExtractorOptions<'a> {
    FeedExtractorOptions {
        feed_link_expr: vec![],
        feed_title_expr: vec!["/rss/channel/dc:title/text()"],
        feed_entries_expr: vec![],
        entry_link_expr: vec![],
        entry_title_expr: vec!["./dc:title/text()"],
        entry_published_expr: vec!["./dc:date/text()"],
        entry_description_expr: vec!["./dc:description/text()"],
        entry_author_expr: vec!["./dc:creator/text()"],
        entry_thumbnail_expr: vec![],
    }
}

pub fn itunes_extractor_options<'a>() -> FeedExtractorOptions<'a> {
    FeedExtractorOptions {
        feed_link_expr: vec![],
        feed_title_expr: vec!["/rss/channel/itunes:title/text()"],
        feed_entries_expr: vec![],
        entry_link_expr: vec![],
        entry_title_expr: vec!["./itunes:title/text()"],
        entry_published_expr: vec![],
        entry_description_expr: vec![],
        entry_author_expr: vec!["/rss/channel/itunes:author/text()"],
        entry_thumbnail_expr: vec!["./itunes:image/@href"],
    }
}
