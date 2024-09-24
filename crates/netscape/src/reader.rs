use std::{collections::BTreeMap, io::BufRead};

use html5gum::{HtmlString, IoReader, Token, Tokenizer};

use crate::{Item, Netscape};

enum NetscapeField {
    Title,
    H1,
    H3,
    A,
}

pub fn from_reader<R: BufRead>(reader: R) -> Result<Netscape, anyhow::Error> {
    let mut netscape = Netscape::default();

    let mut current: Option<NetscapeField> = None;
    let mut stack: Vec<Item> = Vec::new();

    let mut tokenizer = Tokenizer::new(IoReader::new(reader));

    while let Some(Ok(token)) = tokenizer.next() {
        match token {
            Token::StartTag(tag) => match tag.name.as_slice() {
                b"title" => current = Some(NetscapeField::Title),
                b"h1" => current = Some(NetscapeField::H1),
                b"h3" => {
                    current = Some(NetscapeField::H3);

                    let item = parse_attributes(tag.attributes)?;
                    stack.push(item);
                }
                b"a" => {
                    current = Some(NetscapeField::A);

                    let item = parse_attributes(tag.attributes)?;
                    stack.push(item);
                }
                _ => {}
            },
            Token::String(text) => {
                let text = String::from_utf8(text.0)
                    .unwrap()
                    .split_whitespace()
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");

                match current {
                    Some(NetscapeField::Title) => {
                        netscape.title = text;

                        current = None;
                    }
                    Some(NetscapeField::H1) => {
                        netscape.h1 = text;

                        current = None;
                    }
                    Some(NetscapeField::H3) => {
                        if let Some(item) = stack.last_mut() {
                            item.title = text;
                        }

                        current = None
                    }
                    Some(NetscapeField::A) => {
                        if let Some(item) = stack.last_mut() {
                            item.title = text;
                        }

                        current = None
                    }
                    _ => {}
                }
            }
            Token::EndTag(tag) => match tag.name.as_slice() {
                b"a" => {
                    if let Some(item) = stack.pop() {
                        if let Some(parent) = stack.last_mut() {
                            parent.item.get_or_insert_with(Vec::new).push(item);
                        } else {
                            netscape.items.push(item);
                        }
                    }
                }
                b"dl" => {
                    if let Some(item) = stack.pop() {
                        if let Some(parent) = stack.last_mut() {
                            parent.item.get_or_insert_with(Vec::new).push(item);
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

fn parse_attributes(attributes: BTreeMap<HtmlString, HtmlString>) -> Result<Item, anyhow::Error> {
    let mut item = Item::default();

    for (key, value) in attributes {
        match key.0.as_slice() {
            b"href" => item.href = Some(String::from_utf8(value.0)?),
            b"add_date" => item.add_date = Some(String::from_utf8(value.0)?.parse()?),
            b"last_visit" => item.last_visit = Some(String::from_utf8(value.0)?.parse()?),
            b"last_modified" => item.last_modified = Some(String::from_utf8(value.0)?.parse()?),
            _ => {}
        }
    }

    Ok(item)
}
