use std::io::Write;

use crate::{Error, Item, Netscape};

const DOCTYPE: &str = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>"#;
const CONTENT_TYPE: &str = r#"<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">"#;

pub fn to_writer<W: Write>(mut writer: W, netscape: Netscape) -> Result<(), Error> {
    writeln!(writer, "{}", DOCTYPE)?;
    writeln!(writer, "{}", CONTENT_TYPE)?;

    writeln!(writer, "<TITLE>{}</TITLE>", netscape.title)?;
    writeln!(writer, "<H1>{}</H1>", netscape.h1)?;
    writeln!(writer, "<DL><p>")?;

    write_items(&mut writer, &netscape.items, 1)?;

    writeln!(writer, "</DL><p>")?;

    Ok(())
}

fn write_items<W: Write>(writer: &mut W, items: &[Item], level: usize) -> Result<(), Error> {
    for item in items {
        let mut attributes: Vec<String> = Vec::new();
        if let Some(add_date) = item.add_date {
            attributes.push(format!(r#"ADD_DATE="{}""#, add_date));
        }
        if let Some(href) = &item.href {
            attributes.push(format!(r#"HREF="{}""#, href));
        }
        if let Some(last_visit) = item.last_visit {
            attributes.push(format!(r#"LAST_VISIT="{}""#, last_visit));
        }
        if let Some(last_modified) = item.last_modified {
            attributes.push(format!(r#"LAST_MODIFIED="{}""#, last_modified));
        }

        let attributes_str = attributes.join(" ");
        let indent_str = " ".repeat(4).repeat(level);

        if let Some(children) = &item.item {
            if attributes_str.is_empty() {
                writeln!(writer, "{}<DT><H3>{}</H3>", indent_str, item.title)?;
            } else {
                writeln!(
                    writer,
                    "{}<DT><H3 {}>{}</H3>",
                    indent_str, attributes_str, item.title
                )?;
            }
            writeln!(writer, "{}<DL><p>", indent_str)?;

            write_items(writer, children, level + 1)?;

            writeln!(writer, "{}</DL><p>", indent_str)?;
        } else {
            writeln!(
                writer,
                r#"{}<DT><A {}>{}</A>"#,
                indent_str, attributes_str, item.title
            )?;
        }
    }

    Ok(())
}
