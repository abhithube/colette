use crate::Metadata;

#[derive(Debug, Clone)]
pub struct Feed {
    pub title: String,
    pub href: String,
}

impl Metadata {
    pub(crate) fn handle_rss(&mut self, title: String, href: String) {
        self.feeds.push(Feed { title, href });
    }
}
