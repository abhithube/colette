use std::{collections::HashMap, io::BufRead};

use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

use crate::Error;

#[derive(Debug, Clone, Default)]
pub enum Value {
    #[default]
    Null,
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

pub(crate) fn parse_value<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    initial_tag: String,
    initial_value: Value,
) -> Result<Value, Error> {
    let mut tag_stack: Vec<String> = vec![initial_tag];
    let mut value_stack: Vec<Value> = vec![initial_value];

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();
                tag_stack.push(tag);

                let value = handle_properties(reader, &e)?;
                value_stack.push(value);
            }
            Ok(Event::Empty(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

                let value = handle_properties(reader, &e)?;

                if let Some(parent) = value_stack.last_mut() {
                    handle_value(parent, tag, value);
                } else {
                    return Ok(value);
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape()?.into_owned();
                let value = Value::String(text);

                if let (Some(tag), Some(parent)) = (tag_stack.pop(), value_stack.last_mut()) {
                    handle_text(parent, tag, value);
                }
            }
            Ok(Event::End(e)) => {
                let tag = String::from_utf8_lossy(e.name().0).into_owned();

                if let Some(value) = value_stack.pop() {
                    if let Some(parent) = value_stack.last_mut() {
                        handle_value(parent, tag, value);
                    } else {
                        return Ok(value);
                    }
                }
            }
            _ => (),
        }

        buf.clear();
    }
}

pub(crate) fn handle_properties<'a, R: BufRead>(
    reader: &'a Reader<R>,
    e: &'a BytesStart<'a>,
) -> Result<Value, Error> {
    let mut properties = HashMap::new();
    for attribute in e.attributes() {
        let attribute = attribute.map_err(|e| Error::Parse(e.into()))?;

        let k = String::from_utf8_lossy(attribute.key.0).into_owned();
        let v = attribute
            .decode_and_unescape_value(reader.decoder())?
            .into_owned();

        properties.insert(k, Value::String(v));
    }

    let mut value = Value::default();
    if !properties.is_empty() {
        value = Value::Object(properties);
    }

    Ok(value)
}

fn handle_value(parent: &mut Value, tag: String, value: Value) {
    match parent {
        Value::Null => *parent = Value::Object(HashMap::from([(tag, value)])),
        Value::Array(arr) => {
            arr.push(value);
        }
        Value::Object(obj) => {
            if let Some(old) = obj.get_mut(&tag) {
                match old {
                    Value::Array(arr) => arr.push(value),
                    _ => *old = Value::Array(vec![old.clone(), value]),
                }
            } else {
                obj.insert(tag, value);
            }
        }
        _ => {}
    }
}

fn handle_text(parent: &mut Value, tag: String, value: Value) {
    match parent {
        Value::Null => *parent = value,
        Value::Array(arr) => {
            arr.push(value);
        }
        Value::Object(obj) => {
            obj.values().next();

            if let Some(old) = obj.get_mut(&tag) {
                match old {
                    Value::Array(arr) => arr.push(value),
                    _ => *old = Value::Array(vec![old.clone(), value]),
                }
            } else {
                obj.insert(tag, value);
            }
        }
        _ => {}
    }
}
