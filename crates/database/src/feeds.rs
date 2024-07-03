#[derive(Debug)]
pub struct InsertData<'a> {
    pub link: &'a str,
    pub title: &'a str,
    pub url: Option<&'a str>,
}
