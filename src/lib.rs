pub enum Node {
    Document { children: Vec<Node> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_empty_document() {
        let doc = Node::Document { children: vec![] };

        match doc {
            Node::Document { children } => {
                assert_eq!(children.len(), 0);
            }
        }
    }
}
