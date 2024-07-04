use colette_core::{
    feeds::{ExtractedEntry, ExtractedFeed, ExtractorOptions},
    scraper::{extractor::Error, Extractor},
};
use skyscraper::{
    html,
    xpath::{
        self,
        grammar::{data_model::XpathItem, XpathItemTreeNode},
        query,
        xpath_item_set::XpathItemSet,
        XpathItemTree,
    },
};

pub struct DefaultFeedExtractor {
    pub options: ExtractorOptions,
}

impl Extractor<ExtractedFeed> for DefaultFeedExtractor {
    fn extract(&self, url: &str, raw: &str) -> Result<ExtractedFeed, Error> {
        let document = html::parse(raw).map_err(|e| Error(e.into()))?;
        let tree = XpathItemTree::from(&document);

        let entries = query::find(&tree, self.options.feed_entries_expr)
            .ok()
            .map_or(vec![], |set| {
                set.into_iter()
                    .map(|item| ExtractedEntry {
                        link: find_text_from_item(
                            &tree,
                            item.clone(),
                            self.options.entry_link_expr,
                        ),
                        title: find_text_from_item(
                            &tree,
                            item.clone(),
                            self.options.entry_title_expr,
                        ),
                        published: self
                            .options
                            .entry_published_expr
                            .and_then(|e| find_text_from_item(&tree, item.clone(), e)),
                        description: self
                            .options
                            .entry_description_expr
                            .and_then(|e| find_text_from_item(&tree, item.clone(), e)),
                        author: self
                            .options
                            .entry_author_expr
                            .and_then(|e| find_text_from_item(&tree, item.clone(), e)),
                        thumbnail: self
                            .options
                            .entry_thumbnail_expr
                            .and_then(|e| find_text_from_item(&tree, item, e)),
                    })
                    .collect()
            });

        let feed = ExtractedFeed {
            link: self
                .options
                .feed_link_expr
                .and_then(|e| find_text_from_tree(&tree, e))
                .or(Some(url.to_owned())),
            title: find_text_from_tree(&tree, self.options.feed_title_expr),
            entries,
        };

        Ok(feed)
    }
}

fn extract_text(node: &XpathItemTreeNode) -> Option<String> {
    match node {
        XpathItemTreeNode::TextNode(text) => Some(text.content.clone()),
        XpathItemTreeNode::AttributeNode(attr) => Some(attr.value.clone()),
        _ => None,
    }
}

fn handle_result_set(set: XpathItemSet) -> Option<String> {
    set.into_iter()
        .next()
        .and_then(|e| e.as_node().ok().cloned())
        .and_then(extract_text)
}

fn find_text_from_tree(tree: &XpathItemTree, expr: &str) -> Option<String> {
    query::find(tree, expr).ok().and_then(handle_result_set)
}

fn find_text_from_item(tree: &XpathItemTree, item: XpathItem, expr: &str) -> Option<String> {
    xpath::parse(expr)
        .ok()
        .and_then(|e| e.apply_to_item(tree, item).ok().and_then(handle_result_set))
}
