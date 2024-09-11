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
    pub height: Option<i32>,
    pub width: Option<i32>,

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
    pub height: Option<i32>,
    pub width: Option<i32>,

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

pub fn handle_microdata(
    schema_object: &mut SchemaObjectOrValue,
    itemprop: String,
    content: String,
) {
    match schema_object {
        SchemaObjectOrValue::SchemaObject(schema_obj) => match schema_obj {
            SchemaObject::Article(article) => update_article(article, itemprop, content),
            SchemaObject::ImageObject(image_object) => {
                update_image_object(image_object, itemprop, content)
            }
            SchemaObject::Person(person) => update_person(person, itemprop, content),
            SchemaObject::VideoObject(video_object) => {
                update_video_object(video_object, itemprop, content)
            }
            SchemaObject::WebPage(webpage) => update_webpage(webpage, itemprop, content),
            SchemaObject::WebSite(website) => update_website(website, itemprop, content),
        },
        SchemaObjectOrValue::Other(value) => {
            if let Value::Object(object) = value {
                object.insert(itemprop, Value::String(content));
            }
        }
    }
}

fn update_article(article: &mut Article, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Creative Work
        "dateCreated" => article.date_created = Some(content),
        "dateModified" => article.date_modified = Some(content),
        "datePublished" => article.date_published = Some(content),
        "thumbnailUrl" => article.thumbnail_url = Some(content),

        // Thing
        "description" => article.description = Some(content),
        "name" => article.name = Some(content),
        "url" => article.url = Some(content),

        _ => update_additional_properties(&mut article.additional_properties, itemprop, content),
    }
}

fn update_image_object(image_object: &mut ImageObject, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Media Object
        "contentUrl" => image_object.content_url = Some(content),
        "uploadDate" => image_object.upload_date = Some(content),
        "height" => image_object.height = content.parse().ok(),
        "width" => image_object.width = content.parse().ok(),

        // Creative Work
        "dateCreated" => image_object.date_created = Some(content),
        "dateModified" => image_object.date_modified = Some(content),
        "datePublished" => image_object.date_published = Some(content),
        "thumbnailUrl" => image_object.thumbnail_url = Some(content),

        // Thing
        "description" => image_object.description = Some(content),
        "name" => image_object.name = Some(content),
        "url" => image_object.url = Some(content),

        _ => {
            update_additional_properties(&mut image_object.additional_properties, itemprop, content)
        }
    }
}

fn update_person(person: &mut Person, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Thing
        "description" => person.description = Some(content),
        "name" => person.name = Some(content),
        "url" => person.url = Some(content),

        _ => update_additional_properties(&mut person.additional_properties, itemprop, content),
    }
}

fn update_video_object(video_object: &mut VideoObject, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Media Object
        "contentUrl" => video_object.content_url = Some(content),
        "uploadDate" => video_object.upload_date = Some(content),
        "height" => video_object.height = content.parse().ok(),
        "width" => video_object.width = content.parse().ok(),

        // Creative Work
        "dateCreated" => video_object.date_created = Some(content),
        "dateModified" => video_object.date_modified = Some(content),
        "datePublished" => video_object.date_published = Some(content),
        "thumbnailUrl" => video_object.thumbnail_url = Some(content),

        // Thing
        "description" => video_object.description = Some(content),
        "name" => video_object.name = Some(content),
        "url" => video_object.url = Some(content),

        _ => {
            update_additional_properties(&mut video_object.additional_properties, itemprop, content)
        }
    }
}

fn update_webpage(webpage: &mut WebPage, itemprop: String, content: String) {
    match itemprop.as_str() {
        // CreativeWork
        "dateCreated" => webpage.date_created = Some(content),
        "dateModified" => webpage.date_modified = Some(content),
        "datePublished" => webpage.date_published = Some(content),
        "thumbnailUrl" => webpage.thumbnail_url = Some(content),

        // Thing
        "description" => webpage.description = Some(content),
        "name" => webpage.name = Some(content),
        "url" => webpage.url = Some(content),

        _ => update_additional_properties(&mut webpage.additional_properties, itemprop, content),
    }
}

fn update_website(website: &mut WebSite, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Creative Work
        "dateCreated" => website.date_created = Some(content),
        "dateModified" => website.date_modified = Some(content),
        "datePublished" => website.date_published = Some(content),
        "thumbnailUrl" => website.thumbnail_url = Some(content),

        // Thing
        "description" => website.description = Some(content),
        "name" => website.name = Some(content),
        "url" => website.url = Some(content),

        _ => update_additional_properties(&mut website.additional_properties, itemprop, content),
    }
}

fn update_additional_properties(properties: &mut Value, itemprop: String, content: String) {
    if !properties.is_object() {
        *properties = Value::Object(Map::new());
    }

    if let Some(map) = properties.as_object_mut() {
        map.insert(itemprop, Value::String(content));
    }
}
