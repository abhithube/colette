#[derive(Debug, Clone, Default)]
pub struct Basic {
    pub author: Option<String>,
    pub description: Option<String>,
    pub title: Option<String>,
}

pub fn handle_basic(basic: &mut Basic, name: String, content: String) {
    match name.as_str() {
        "auth" => {
            basic.author = Some(content);
        }
        "description" => {
            basic.description = Some(content);
        }
        "title" => {
            basic.title = Some(content);
        }
        _ => {}
    }
}
