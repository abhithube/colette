use colette_core::feeds::FeedsCreateData;

#[derive(Clone, Debug)]
pub struct InsertParams<'a> {
    pub link: &'a str,
    pub title: &'a str,
    pub url: Option<&'a str>,
}

impl<'a> From<&'a FeedsCreateData> for InsertParams<'a> {
    fn from(value: &'a FeedsCreateData) -> Self {
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
