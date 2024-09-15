use std::io::Write;

use quick_xml::{events::BytesText, Writer};

use crate::{Opml, Outline, OutlineType, Version};

pub fn to_writer<W: Write>(mut writer: W, opml: Opml) -> Result<(), anyhow::Error> {
    writer.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>"#)?;

    let mut writer = Writer::new(writer);

    writer
        .create_element("opml")
        .with_attribute((
            "version",
            match opml.version {
                Version::V1 => "1.0",
                Version::V1_1 => "1.1",
                Version::V2 => "2.0",
            },
        ))
        .write_inner_content(|writer| {
            let _ = writer
                .create_element("head")
                .write_inner_content(|writer| {
                    let _ = writer
                        .create_element("title")
                        .write_text_content(BytesText::new(&opml.head.title))?;

                    Ok::<(), quick_xml::Error>(())
                })?;

            let _ = writer
                .create_element("body")
                .write_inner_content(|writer| {
                    fn write_outlines<W: Write>(
                        writer: &mut Writer<W>,
                        outlines: Vec<Outline>,
                    ) -> Result<(), quick_xml::Error> {
                        for outline in outlines {
                            if let Some(children) = outline.outline {
                                let mut element_writer = writer
                                    .create_element("outline")
                                    .with_attribute(("text", outline.text.as_str()));

                                if let Some(title) = outline.title.as_deref() {
                                    element_writer =
                                        element_writer.with_attribute(("title", title));
                                }

                                element_writer.write_inner_content(|writer| {
                                    write_outlines(writer, children)
                                })?;
                            } else {
                                let mut element_writer = writer
                                    .create_element("outline")
                                    .with_attribute(("text", outline.text.as_str()));

                                if let Some(r#type) = &outline.r#type {
                                    match r#type {
                                        OutlineType::Rss => {
                                            element_writer =
                                                element_writer.with_attribute(("type", "rss"));
                                        }
                                    }
                                }
                                if let Some(xml_url) = outline.xml_url.as_deref() {
                                    element_writer =
                                        element_writer.with_attribute(("xmlUrl", xml_url));
                                }
                                if let Some(title) = outline.title.as_deref() {
                                    element_writer =
                                        element_writer.with_attribute(("title", title));
                                }
                                if let Some(html_url) = outline.html_url.as_deref() {
                                    element_writer =
                                        element_writer.with_attribute(("htmlUrl", html_url));
                                }

                                element_writer.write_empty()?;
                            }
                        }

                        Ok(())
                    }

                    write_outlines(writer, opml.body.outlines)?;

                    Ok::<(), quick_xml::Error>(())
                })?;

            Ok::<(), quick_xml::Error>(())
        })?;

    Ok(())
}
