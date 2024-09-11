use std::cell::{Cell, RefCell};

use basic::handle_basic;
pub use basic::Basic;
use html5ever::{
    tendril::StrTendril,
    tokenizer::{
        CharacterTokens, EndTag, StartTag, Tag, TagToken, Token, TokenSink, TokenSinkResult,
    },
};
use open_graph::handle_open_graph;
pub use open_graph::OpenGraph;
use rss::handle_rss;
pub use rss::Feed;
use schema_org::{handle_json_ld, SchemaObjectOrValue};

mod basic;
mod open_graph;
mod rss;
mod schema_org;

#[derive(Debug, Clone, Default)]
pub struct MetadataSink {
    pub(crate) basic: RefCell<Basic>,
    pub(crate) feeds: RefCell<Vec<Feed>>,
    pub(crate) open_graph: RefCell<Option<OpenGraph>>,
    pub(crate) schema_org: RefCell<Vec<SchemaObjectOrValue>>,
    in_ld_json: Cell<bool>,
    inner_text: RefCell<StrTendril>,
}

impl TokenSink for MetadataSink {
    type Handle = ();

    fn process_token(&self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            CharacterTokens(inner_text) => self.handle_inner_text(inner_text),
            TagToken(tag) => match tag.kind {
                StartTag => self.handle_start_tag(tag),
                EndTag => self.handle_end_tag(tag),
            },
            _ => {}
        }

        TokenSinkResult::Continue
    }
}

impl MetadataSink {
    fn handle_start_tag(&self, tag: Tag) {
        let mut content: Option<StrTendril> = None;
        let mut href: Option<StrTendril> = None;
        let mut name: Option<StrTendril> = None;
        let mut property: Option<StrTendril> = None;
        let mut title: Option<StrTendril> = None;
        let mut r#type: Option<StrTendril> = None;

        for attr in tag.attrs {
            match attr.name.local.as_ref() {
                "content" => content = Some(attr.value),
                "href" => href = Some(attr.value),
                "name" => name = Some(attr.value),
                "property" => property = Some(attr.value),
                "title" => title = Some(attr.value),
                "type" => r#type = Some(attr.value),
                _ => {}
            }
        }

        match (content, href, name, property, title) {
            (Some(content), _, Some(name), _, _) if tag.name.as_ref() == "meta" => {
                let mut basic = self.basic.borrow_mut();
                handle_basic(&mut basic, name.into(), content.into());
            }
            (_, Some(href), _, _, Some(title))
                if tag.name.as_ref() == "link"
                    && r#type.as_deref() == Some("application/rss+xml") =>
            {
                handle_rss(&mut self.feeds.borrow_mut(), title.into(), href.into());
            }
            (Some(content), _, _, Some(mut property), _) if property.contains(":") => {
                let mut open_graph = self.open_graph.borrow_mut();
                let open_graph = open_graph.get_or_insert_with(OpenGraph::default);

                if let Some((_, suffix)) = property.split_once(':') {
                    property = suffix.into();
                }

                handle_open_graph(open_graph, property.into(), content.into());
            }
            _ if tag.name.as_ref() == "script"
                && r#type.as_deref() == Some("application/ld+json") =>
            {
                self.in_ld_json.set(true);
            }
            _ => {}
        }
    }

    fn handle_inner_text(&self, inner_text: StrTendril) {
        if self.in_ld_json.get() {
            let mut ld_json = self.inner_text.borrow_mut();
            ld_json.push_tendril(&inner_text);
        }
    }

    fn handle_end_tag(&self, tag: Tag) {
        if tag.name.as_ref() == "script" && self.in_ld_json.get() {
            let text = self.inner_text.take();
            let mut schema_org = self.schema_org.borrow_mut();

            handle_json_ld(&mut schema_org, text.into());

            self.in_ld_json.set(false);
        }
    }
}
