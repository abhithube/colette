use std::io::Write;

use crate::{Error, Opml, Outline};

const HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>"#;

pub fn to_writer<W: Write>(mut writer: W, opml: Opml) -> Result<(), Error> {
    writeln!(writer, "{HEADER}")?;

    writeln!(writer, r#"<opml version="{}">"#, opml.version)?;

    writeln!(writer, "{}<head>", " ".repeat(4))?;
    writeln!(
        writer,
        "{}<title>{}</title>",
        " ".repeat(4).repeat(2),
        quick_xml::escape::escape(opml.head.title)
    )?;
    writeln!(writer, "{}</head>", " ".repeat(4))?;

    writeln!(writer, "{}<body>", " ".repeat(4))?;

    write_outlines(&mut writer, &opml.body.outlines, 2)?;

    writeln!(writer, "{}</body>", " ".repeat(4))?;

    writeln!(writer, "</opml>")?;

    Ok(())
}

fn write_outlines<W: Write>(
    writer: &mut W,
    outlines: &[Outline],
    level: usize,
) -> Result<(), Error> {
    for outline in outlines {
        let mut attributes: Vec<String> = Vec::new();
        if let Some(ref r#type) = outline.r#type {
            attributes.push(format!(r#"type="{type}""#));
        }
        attributes.push(format!(
            r#"text="{}""#,
            quick_xml::escape::escape(&outline.text)
        ));
        if let Some(ref xml_url) = outline.xml_url {
            attributes.push(format!(r#"xmlUrl="{xml_url}""#));
        }
        if let Some(ref title) = outline.title {
            attributes.push(format!(r#"title="{}""#, quick_xml::escape::escape(title)));
        }
        if let Some(ref html_url) = outline.html_url {
            attributes.push(format!(r#"htmlUrl="{html_url}""#));
        }

        let attributes_str = attributes.join(" ");
        let indent_str = " ".repeat(4).repeat(level);

        if !outline.outline.is_empty() {
            writeln!(writer, "{indent_str}<outline {attributes_str}>")?;

            write_outlines(writer, &outline.outline, level + 1)?;

            writeln!(writer, "{indent_str}</outline>")?;
        } else {
            writeln!(writer, r#"{indent_str}<outline {attributes_str}/>"#)?;
        }
    }

    Ok(())
}
