use colette_core::feeds::FeedCreateData;

#[derive(Debug)]
pub struct InsertData<'a> {
    pub link: &'a str,
    pub title: &'a str,
    pub url: Option<&'a str>,
}

impl<'a> From<&'a FeedCreateData<'_>> for InsertData<'a> {
    fn from(value: &'a FeedCreateData<'_>) -> Self {
        let link = value.feed.link.as_str();

        Self {
            link,
            title: value.feed.title,
            url: if value.url == link {
                None
            } else {
                Some(value.url)
            },
        }
    }
}
