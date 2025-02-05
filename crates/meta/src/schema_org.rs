use crate::util::Value;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub(crate) enum SchemaOrg {
    Graph {
        #[serde(rename = "@graph")]
        graph: Vec<SchemaObjectOrValue>,
    },
    Single(SchemaObjectOrValue),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
#[serde(untagged)]
pub enum SchemaObjectOrValue {
    SchemaObject(SchemaObject),
    Other(Value),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
#[serde(tag = "@type")]
pub enum SchemaObject {
    Article(Article),
    WebPage(WebPage),
    VideoObject(VideoObject),
    WebSite(WebSite),
    ImageObject(ImageObject),
    Person(Person),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    // Thing
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub image: Option<ImageObject>,

    // CreativeWork
    pub author: Option<Person>,
    pub date_published: Option<String>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub thumbnail: Option<ImageObject>,
    pub thumbnail_url: Option<String>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebPage {
    // Thing
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub image: Option<ImageObject>,

    // CreativeWork
    pub author: Option<Person>,
    pub date_published: Option<String>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub thumbnail: Option<ImageObject>,
    pub thumbnail_url: Option<String>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoObject {
    // Thing
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub image: Option<ImageObject>,

    // CreativeWork
    pub author: Option<Person>,
    pub date_published: Option<String>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub thumbnail: Option<ImageObject>,
    pub thumbnail_url: Option<String>,

    // Media Object
    pub content_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub upload_date: Option<String>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSite {
    // Thing
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub image: Option<ImageObject>,

    // CreativeWork
    pub author: Option<Person>,
    pub date_published: Option<String>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub thumbnail: Option<ImageObject>,
    pub thumbnail_url: Option<String>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
    // Thing
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub image: Option<Box<ImageObject>>,

    // CreativeWork
    pub author: Option<Box<Person>>,
    pub date_published: Option<String>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub thumbnail: Option<Box<ImageObject>>,
    pub thumbnail_url: Option<String>,

    // Media Object
    pub content_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub upload_date: Option<String>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    // Thing
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub image: Option<ImageObject>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

pub(crate) fn handle_json_ld(schema_org: &mut Vec<SchemaObjectOrValue>, text: String) {
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

pub(crate) fn handle_microdata(
    schema_org: &mut SchemaObjectOrValue,
    itemprop: String,
    content: String,
) {
    match schema_org {
        SchemaObjectOrValue::SchemaObject(schema_obj) => match *schema_obj {
            SchemaObject::Article(ref mut article) => update_article(article, itemprop, content),
            SchemaObject::ImageObject(ref mut image_object) => {
                update_image_object(image_object, itemprop, content)
            }
            SchemaObject::Person(ref mut person) => update_person(person, itemprop, content),
            SchemaObject::VideoObject(ref mut video_object) => {
                update_video_object(video_object, itemprop, content)
            }
            SchemaObject::WebPage(ref mut webpage) => update_webpage(webpage, itemprop, content),
            SchemaObject::WebSite(ref mut website) => update_website(website, itemprop, content),
        },
        SchemaObjectOrValue::Other(Value::Object(object)) => {
            let value = object.entry(itemprop).or_insert(Value::Array(Vec::new()));
            if let Value::Array(array) = value {
                array.push(Value::String(content));
            }
        }
        _ => {}
    }
}

fn update_article(article: &mut Article, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Thing
        "name" => article.name = Some(content),
        "url" => article.url = Some(content),
        "description" => article.description = Some(content),

        // Creative Work
        "datePublished" => article.date_published = Some(content),
        "dateCreated" => article.date_created = Some(content),
        "dateModified" => article.date_modified = Some(content),
        "thumbnailUrl" => article.thumbnail_url = Some(content),

        _ => update_additional_properties(&mut article.additional_properties, itemprop, content),
    }
}

fn update_webpage(webpage: &mut WebPage, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Thing
        "name" => webpage.name = Some(content),
        "url" => webpage.url = Some(content),
        "description" => webpage.description = Some(content),

        // Creative Work
        "datePublished" => webpage.date_published = Some(content),
        "dateCreated" => webpage.date_created = Some(content),
        "dateModified" => webpage.date_modified = Some(content),
        "thumbnailUrl" => webpage.thumbnail_url = Some(content),

        _ => update_additional_properties(&mut webpage.additional_properties, itemprop, content),
    }
}

fn update_video_object(video_object: &mut VideoObject, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Thing
        "name" => video_object.name = Some(content),
        "url" => video_object.url = Some(content),
        "description" => video_object.description = Some(content),

        // Creative Work
        "datePublished" => video_object.date_published = Some(content),
        "dateCreated" => video_object.date_created = Some(content),
        "dateModified" => video_object.date_modified = Some(content),
        "thumbnailUrl" => video_object.thumbnail_url = Some(content),

        // Media Object
        "contentUrl" => video_object.content_url = Some(content),
        "width" => video_object.width = content.parse().ok(),
        "height" => video_object.height = content.parse().ok(),
        "uploadDate" => video_object.upload_date = Some(content),

        _ => {
            update_additional_properties(&mut video_object.additional_properties, itemprop, content)
        }
    }
}

fn update_website(website: &mut WebSite, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Thing
        "name" => website.name = Some(content),
        "url" => website.url = Some(content),
        "description" => website.description = Some(content),

        // Creative Work
        "datePublished" => website.date_published = Some(content),
        "dateCreated" => website.date_created = Some(content),
        "dateModified" => website.date_modified = Some(content),
        "thumbnailUrl" => website.thumbnail_url = Some(content),

        _ => update_additional_properties(&mut website.additional_properties, itemprop, content),
    }
}

fn update_image_object(image_object: &mut ImageObject, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Thing
        "name" => image_object.name = Some(content),
        "url" => image_object.url = Some(content),
        "description" => image_object.description = Some(content),

        // Creative Work
        "datePublished" => image_object.date_published = Some(content),
        "dateCreated" => image_object.date_created = Some(content),
        "dateModified" => image_object.date_modified = Some(content),
        "thumbnailUrl" => image_object.thumbnail_url = Some(content),

        // Media Object
        "contentUrl" => image_object.content_url = Some(content),
        "width" => image_object.width = content.parse().ok(),
        "height" => image_object.height = content.parse().ok(),
        "uploadDate" => image_object.upload_date = Some(content),

        _ => {
            update_additional_properties(&mut image_object.additional_properties, itemprop, content)
        }
    }
}

fn update_person(person: &mut Person, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Thing
        "name" => person.name = Some(content),
        "url" => person.url = Some(content),
        "description" => person.description = Some(content),

        _ => update_additional_properties(&mut person.additional_properties, itemprop, content),
    }
}

fn update_additional_properties(properties: &mut Value, itemprop: String, content: String) {
    if let Value::Object(object) = properties {
        let value = object.entry(itemprop).or_insert(Value::Array(Vec::new()));
        if let Value::Array(array) = value {
            array.push(Value::String(content));
        }
    }
}
