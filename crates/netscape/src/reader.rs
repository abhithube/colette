use core::str;
use std::{collections::BTreeMap, io::Read};

use html5gum::{HtmlString, IoReader, Token, Tokenizer};

use crate::{Error, Item, Netscape, ParseError};

#[derive(Debug, Clone)]
enum StartTag {
    Title,
    H1,
    H3,
    A,
}

pub fn from_reader<R: Read>(reader: R) -> Result<Netscape, Error> {
    let mut netscape = Netscape::default();

    let mut tag_stack: Vec<StartTag> = Vec::new();
    let mut item_stack: Vec<Item> = Vec::new();

    let mut tokenizer = Tokenizer::new(IoReader::new(reader));

    while let Some(Ok(token)) = tokenizer.next() {
        match token {
            Token::StartTag(tag) => match tag.name.as_slice() {
                b"title" => tag_stack.push(StartTag::Title),
                b"h1" => tag_stack.push(StartTag::H1),
                b"h3" => {
                    let item = parse_attributes(tag.attributes)?;
                    item_stack.push(item);

                    tag_stack.push(StartTag::H3)
                }
                b"a" => {
                    let item = parse_attributes(tag.attributes)?;
                    item_stack.push(item);

                    tag_stack.push(StartTag::A)
                }
                _ => {}
            },
            Token::String(text) => {
                let text = str::from_utf8(&text.0)
                    .map_err(ParseError::Utf)?
                    .split_whitespace()
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");

                match tag_stack.pop() {
                    Some(StartTag::Title) => {
                        netscape.title = text;
                    }
                    Some(StartTag::H1) => {
                        netscape.h1 = text;
                    }
                    Some(StartTag::H3) => {
                        if let Some(item) = item_stack.last_mut() {
                            item.title = text;
                        }
                    }
                    Some(StartTag::A) => {
                        if let Some(item) = item_stack.last_mut() {
                            item.title = text;
                        }
                    }
                    None => {}
                }
            }
            Token::EndTag(tag) => match tag.name.as_slice() {
                b"a" | b"dl" => {
                    if let Some(item) = item_stack.pop() {
                        if let Some(parent) = item_stack.last_mut() {
                            parent.item.push(item);
                        } else {
                            netscape.items.push(item);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    Ok(netscape)
}

fn parse_attributes(attributes: BTreeMap<HtmlString, HtmlString>) -> Result<Item, Error> {
    let mut item = Item::default();

    for (key, value) in attributes {
        match key.0.as_slice() {
            b"href" => {
                item.href = Some(
                    String::from_utf8(value.0)
                        .map_err(|e| e.utf8_error())
                        .map_err(ParseError::Utf)?,
                )
            }
            b"add_date" => {
                item.add_date = Some(
                    str::from_utf8(&value.0)
                        .map_err(ParseError::Utf)?
                        .parse()
                        .map_err(ParseError::Int)?,
                )
            }
            b"last_visit" => {
                item.last_visit = Some(
                    str::from_utf8(&value.0)
                        .map_err(ParseError::Utf)?
                        .parse()
                        .map_err(ParseError::Int)?,
                )
            }
            b"last_modified" => {
                item.last_modified = Some(
                    str::from_utf8(&value.0)
                        .map_err(ParseError::Utf)?
                        .parse()
                        .map_err(ParseError::Int)?,
                )
            }
            _ => {}
        }
    }

    Ok(item)
}
