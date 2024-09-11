use std::cell::RefCell;

use basic::{handle_basic, Basic};
use html5ever::{
    tendril::StrTendril,
    tokenizer::{
        CharacterTokens, EndTag, StartTag, Tag, TagToken, Token, TokenSink, TokenSinkResult,
    },
};

mod basic;

#[derive(Debug, Clone, Default)]
pub struct MetadataSink {
    pub(crate) basic: RefCell<Basic>,
}

impl TokenSink for MetadataSink {
    type Handle = ();

    fn process_token(&self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            CharacterTokens(_) => {}
            TagToken(tag) => match tag.kind {
                StartTag => self.handle_start_tag(tag),
                EndTag => {}
            },
            _ => {}
        }

        TokenSinkResult::Continue
    }
}

impl MetadataSink {
    fn handle_start_tag(&self, tag: Tag) {
        let mut content: Option<StrTendril> = None;
        let mut name: Option<StrTendril> = None;

        for attr in tag.attrs {
            match attr.name.local.as_ref() {
                "content" => content = Some(attr.value),
                "name" => name = Some(attr.value),
                _ => {}
            }
        }

        match (content, name) {
            (Some(content), Some(name)) if tag.name.as_ref() == "meta" => {
                let mut basic = self.basic.borrow_mut();
                handle_basic(&mut basic, name.into(), content.into());
            }
            _ => {}
        }
    }
}
