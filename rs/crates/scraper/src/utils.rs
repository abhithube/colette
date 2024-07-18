use libxml::{tree::Node, xpath::Context};

pub trait Xpath {
    fn find_first_content(&mut self, exprs: &[&str], node: Option<&Node>) -> Option<String>;

    fn find_nodes(&mut self, exprs: &[&str], node: Option<&Node>) -> Vec<Node>;
}

impl Xpath for Context {
    fn find_first_content(&mut self, exprs: &[&str], node: Option<&Node>) -> Option<String> {
        exprs
            .iter()
            .find_map(|expr| self.findvalue(expr, node).ok().filter(|e| !e.is_empty()))
    }

    fn find_nodes(&mut self, exprs: &[&str], node: Option<&Node>) -> Vec<Node> {
        exprs
            .iter()
            .find_map(|expr| self.findnodes(expr, node).ok())
            .unwrap_or(vec![])
    }
}
