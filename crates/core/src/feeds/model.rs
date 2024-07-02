#[derive(Debug)]
pub struct ExtractorOptions {
    pub feed_link_expr: Option<&'static str>,
    pub feed_title_expr: &'static str,
    pub feed_entries_expr: &'static str,
    pub entry_link_expr: &'static str,
    pub entry_title_expr: &'static str,
    pub entry_published_expr: Option<&'static str>,
    pub entry_description_expr: Option<&'static str>,
    pub entry_author_expr: Option<&'static str>,
    pub entry_thumbnail_expr: Option<&'static str>,
}

#[derive(Debug)]
pub struct ExtractedFeed {
    pub link: Option<String>,
    pub title: Option<String>,
    pub entries: Vec<ExtractedEntry>,
}

#[derive(Debug)]
pub struct ExtractedEntry {
    pub link: Option<String>,
    pub title: Option<String>,
    pub published: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail: Option<String>,
}
