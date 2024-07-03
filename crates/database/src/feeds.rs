use colette_core::feeds::FeedCreateData;

#[derive(Debug)]
pub struct InsertData<'a> {
    pub link: &'a str,
    pub title: &'a str,
    pub url: Option<&'a str>,
}

impl<'a> From<&'a FeedCreateData> for InsertData<'a> {
    fn from(value: &'a FeedCreateData) -> Self {
        let link = value.feed.link.as_str();
        let url = value.url.as_str();

        Self {
            link,
            title: value.feed.title.as_str(),
            url: if url == link { None } else { Some(url) },
        }
    }
}
