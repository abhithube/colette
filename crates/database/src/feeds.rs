use colette_core::feeds::FeedCreateData;

#[derive(Clone, Debug)]
pub struct InsertParams<'a> {
    pub link: &'a str,
    pub title: &'a str,
    pub url: Option<&'a str>,
}

impl<'a> From<&'a FeedCreateData> for InsertParams<'a> {
    fn from(value: &'a FeedCreateData) -> Self {
        let link = value.feed.link.as_str();

        Self {
            link,
            title: &value.feed.title,
            url: if value.url == link {
                None
            } else {
                Some(&value.url)
            },
        }
    }
}
