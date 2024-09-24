pub use reader::from_reader;
pub use writer::to_writer;

mod reader;
mod writer;

#[derive(Clone, Debug)]
pub struct Netscape {
    pub title: String,
    pub h1: String,
    pub items: Vec<Item>,
}

impl Default for Netscape {
    fn default() -> Self {
        let title = "Bookmarks".to_owned();

        Self {
            title: title.clone(),
            h1: title,
            items: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Item {
    pub title: String,
    pub add_date: Option<i64>,
    pub last_modified: Option<i64>,
    pub href: Option<String>,
    pub last_visit: Option<i64>,
    pub item: Option<Vec<Item>>,
}
