use crate::Metadata;

#[derive(Debug, Clone, Default)]
pub struct Basic {
    pub author: Option<String>,
    pub description: Option<String>,
    pub title: Option<String>,
}

impl Metadata {
    pub(crate) fn handle_basic(&mut self, name: &str, content: String) {
        match name {
            "auth" => {
                self.basic.author = Some(content);
            }
            "description" => {
                self.basic.description = Some(content);
            }
            "title" => {
                self.basic.title = Some(content);
            }
            _ => {}
        }
    }
}
