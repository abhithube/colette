#[derive(Debug, Clone)]
pub struct Feed {
    pub title: String,
    pub href: String,
}

pub fn handle_rss(feeds: &mut Vec<Feed>, title: String, href: String) {
    feeds.push(Feed { title, href });
}
