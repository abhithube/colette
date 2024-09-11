use serde_json::{Map, Value};

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum SchemaOrg {
    Graph {
        #[serde(rename = "@graph")]
        graph: Vec<SchemaObjectOrValue>,
    },
    Single(SchemaObjectOrValue),
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum SchemaObjectOrValue {
    SchemaObject(SchemaObject),
    Other(Value),
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum SchemaObject {
    Article(Article),
    ImageObject(ImageObject),
    Person(Person),
    VideoObject(VideoObject),
    WebPage(WebPage),
    WebSite(WebSite),
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    // CreativeWork
    pub author: Option<Person>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub date_published: Option<String>,
    pub thumbnail: Option<ImageObject>,
    pub thumbnail_url: Option<String>,

    // Thing
    pub description: Option<String>,
    pub image: Option<ImageObject>,
    pub name: Option<String>,
    pub url: Option<String>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
    // Media Object
    pub content_url: Option<String>,
    pub upload_date: Option<String>,

    // CreativeWork
    pub author: Option<Box<Person>>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub date_published: Option<String>,
    pub thumbnail: Option<Box<ImageObject>>,
    pub thumbnail_url: Option<String>,

    // Thing
    pub description: Option<String>,
    pub image: Option<Box<ImageObject>>,
    pub name: Option<String>,
    pub url: Option<String>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    // Thing
    pub description: Option<String>,
    pub image: Option<ImageObject>,
    pub name: Option<String>,
    pub url: Option<String>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoObject {
    // MediaObject
    pub content_url: Option<String>,
    pub upload_date: Option<String>,

    // CreativeWork
    pub author: Option<Person>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub date_published: Option<String>,
    pub thumbnail: Option<ImageObject>,
    pub thumbnail_url: Option<String>,

    // Thing
    pub description: Option<String>,
    pub image: Option<ImageObject>,
    pub name: Option<String>,
    pub url: Option<String>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebPage {
    // Creative Work
    pub author: Option<Person>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub date_published: Option<String>,
    pub thumbnail: Option<ImageObject>,
    pub thumbnail_url: Option<String>,

    // Thing
    pub description: Option<String>,
    pub image: Option<ImageObject>,
    pub name: Option<String>,
    pub url: Option<String>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSite {
    // CreativeWork
    pub author: Option<Person>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub date_published: Option<String>,
    pub thumbnail: Option<ImageObject>,
    pub thumbnail_url: Option<String>,

    // Thing
    pub description: Option<String>,
    pub image: Option<ImageObject>,
    pub name: Option<String>,
    pub url: Option<String>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

pub fn handle_json_ld(schema_org: &mut Vec<SchemaObjectOrValue>, text: String) {
    if let Ok(schema) = serde_json::from_str::<SchemaOrg>(text.as_ref()) {
        match schema {
            SchemaOrg::Graph { mut graph } => {
                schema_org.append(&mut graph);
            }
            SchemaOrg::Single(schema) => {
                schema_org.push(schema);
            }
        }
    }
}
