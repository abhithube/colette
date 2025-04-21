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
    SocialMediaPosting(SocialMediaPosting),
    WebPage(WebPage),
    VideoObject(VideoObject),
    WebSite(WebSite),
    ImageObject(ImageObject),
    Person(Person),
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    // Thing
    pub description: Option<String>,
    pub image: Option<ImageObject>,
    pub name: Option<String>,
    pub url: Option<String>,

    // CreativeWork
    pub author: Option<TypeOrString<Person>>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub date_published: Option<String>,
    pub headline: Option<String>,
    pub thumbnail: Option<ImageObject>,
    pub thumbnail_url: Option<String>,
    pub video: Option<VideoObject>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialMediaPosting {
    // Thing
    pub description: Option<String>,
    pub image: Option<ImageObject>,
    pub name: Option<String>,
    pub url: Option<String>,

    // CreativeWork
    pub author: Option<TypeOrString<Person>>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub date_published: Option<String>,
    pub headline: Option<String>,
    pub thumbnail: Option<ImageObject>,
    pub thumbnail_url: Option<String>,
    pub video: Option<VideoObject>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebPage {
    // Thing
    pub description: Option<String>,
    pub image: Option<ImageObject>,
    pub name: Option<String>,
    pub url: Option<String>,

    // CreativeWork
    pub author: Option<TypeOrString<Person>>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub date_published: Option<String>,
    pub headline: Option<String>,
    pub thumbnail: Option<ImageObject>,
    pub thumbnail_url: Option<String>,
    pub video: Option<VideoObject>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoObject {
    // Thing
    pub description: Option<String>,
    pub image: Option<ImageObject>,
    pub name: Option<String>,
    pub url: Option<String>,

    // CreativeWork
    pub author: Option<TypeOrString<Person>>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub date_published: Option<String>,
    pub headline: Option<String>,
    pub thumbnail: Option<ImageObject>,
    pub thumbnail_url: Option<String>,

    // Media Object
    pub content_url: Option<String>,
    pub height: Option<i32>,
    pub upload_date: Option<String>,
    pub width: Option<i32>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSite {
    // Thing
    pub description: Option<String>,
    pub image: Option<ImageObject>,
    pub name: Option<String>,
    pub url: Option<String>,

    // CreativeWork
    pub author: Option<Person>,
    pub date_modified: Option<String>,
    pub date_published: Option<String>,
    pub date_created: Option<String>,
    pub headline: Option<String>,
    pub thumbnail: Option<ImageObject>,
    pub thumbnail_url: Option<String>,

    #[serde(flatten)]
    pub additional_properties: Value,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
    // Thing
    pub description: Option<String>,
    pub image: Option<Box<ImageObject>>,
    pub name: Option<String>,
    pub url: Option<String>,

    // CreativeWork
    pub author: Option<Box<TypeOrString<Person>>>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
    pub date_published: Option<String>,
    pub headline: Option<String>,
    pub thumbnail: Option<Box<ImageObject>>,
    pub thumbnail_url: Option<String>,

    // Media Object
    pub content_url: Option<String>,
    pub height: Option<i32>,
    pub upload_date: Option<String>,
    pub width: Option<i32>,

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

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum TypeOrString<T> {
    Type(T),
    String(String),
}

pub(crate) fn handle_json_ld(schema_org: &mut Vec<SchemaObjectOrValue>, text: String) {
    if let Ok(schema) = serde_json::from_str::<SchemaOrg>(&text) {
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
        SchemaObjectOrValue::SchemaObject(schema_obj) => match schema_obj {
            SchemaObject::Article(article) => update_article(article, itemprop, content),
            SchemaObject::SocialMediaPosting(article) => {
                update_social_media_posting(article, itemprop, content)
            }
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
        "description" => article.description = Some(content),
        "name" => article.name = Some(content),
        "url" => article.url = Some(content),

        // Creative Work
        "dateCreated" => article.date_created = Some(content),
        "dateModified" => article.date_modified = Some(content),
        "datePublished" => article.date_published = Some(content),
        "headline" => article.headline = Some(content),
        "thumbnailUrl" => article.thumbnail_url = Some(content),

        _ => update_additional_properties(&mut article.additional_properties, itemprop, content),
    }
}

fn update_social_media_posting(smp: &mut SocialMediaPosting, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Thing
        "description" => smp.description = Some(content),
        "name" => smp.name = Some(content),
        "url" => smp.url = Some(content),

        // Creative Work
        "dateCreated" => smp.date_created = Some(content),
        "dateModified" => smp.date_modified = Some(content),
        "datePublished" => smp.date_published = Some(content),
        "headline" => smp.headline = Some(content),
        "thumbnailUrl" => smp.thumbnail_url = Some(content),

        _ => update_additional_properties(&mut smp.additional_properties, itemprop, content),
    }
}

fn update_webpage(webpage: &mut WebPage, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Thing
        "description" => webpage.description = Some(content),
        "name" => webpage.name = Some(content),
        "url" => webpage.url = Some(content),

        // Creative Work
        "dateCreated" => webpage.date_created = Some(content),
        "dateModified" => webpage.date_modified = Some(content),
        "datePublished" => webpage.date_published = Some(content),
        "headline" => webpage.headline = Some(content),
        "thumbnailUrl" => webpage.thumbnail_url = Some(content),

        _ => update_additional_properties(&mut webpage.additional_properties, itemprop, content),
    }
}

fn update_video_object(video_object: &mut VideoObject, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Thing
        "description" => video_object.description = Some(content),
        "name" => video_object.name = Some(content),
        "url" => video_object.url = Some(content),

        // Creative Work
        "dateCreated" => video_object.date_created = Some(content),
        "dateModified" => video_object.date_modified = Some(content),
        "datePublished" => video_object.date_published = Some(content),
        "headline" => video_object.headline = Some(content),
        "thumbnailUrl" => video_object.thumbnail_url = Some(content),

        // Media Object
        "contentUrl" => video_object.content_url = Some(content),
        "height" => video_object.height = content.parse().ok(),
        "uploadDate" => video_object.upload_date = Some(content),
        "width" => video_object.width = content.parse().ok(),

        _ => {
            update_additional_properties(&mut video_object.additional_properties, itemprop, content)
        }
    }
}

fn update_website(website: &mut WebSite, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Thing
        "description" => website.description = Some(content),
        "name" => website.name = Some(content),
        "url" => website.url = Some(content),

        // Creative Work
        "dateCreated" => website.date_created = Some(content),
        "dateModified" => website.date_modified = Some(content),
        "datePublished" => website.date_published = Some(content),
        "headline" => website.headline = Some(content),
        "thumbnailUrl" => website.thumbnail_url = Some(content),

        _ => update_additional_properties(&mut website.additional_properties, itemprop, content),
    }
}

fn update_image_object(image_object: &mut ImageObject, itemprop: String, content: String) {
    match itemprop.as_str() {
        // Thing
        "description" => image_object.description = Some(content),
        "name" => image_object.name = Some(content),
        "url" => image_object.url = Some(content),

        // Creative Work
        "dateCreated" => image_object.date_created = Some(content),
        "dateModified" => image_object.date_modified = Some(content),
        "datePublished" => image_object.date_published = Some(content),
        "headline" => image_object.headline = Some(content),
        "thumbnailUrl" => image_object.thumbnail_url = Some(content),

        // Media Object
        "contentUrl" => image_object.content_url = Some(content),
        "height" => image_object.height = content.parse().ok(),
        "uploadDate" => image_object.upload_date = Some(content),
        "width" => image_object.width = content.parse().ok(),

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
