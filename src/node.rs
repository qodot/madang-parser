pub enum Node {
    Document { children: Vec<Node> },
    Heading { level: u8, children: Vec<Node> },
    Paragraph { children: Vec<Node> },
    Text(String),
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
            _ => panic!("Expected Document"),
        }
    }

    #[test]
    fn create_text_node() {
        let text = Node::Text(String::from("Hello"));

        match text {
            Node::Text(s) => {
                assert_eq!(s, "Hello");
            }
            _ => panic!("Expected Text"),
        }
    }

    #[test]
    fn document_with_text() {
        let doc = Node::Document {
            children: vec![Node::Text(String::from("안녕하세요"))],
        };

        match doc {
            Node::Document { children } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Text(s) => assert_eq!(s, "안녕하세요"),
                    _ => panic!("Expected Text"),
                }
            }
            _ => panic!("Expected Document"),
        }
    }

    #[test]
    fn document_with_paragraph() {
        let doc = Node::Document {
            children: vec![Node::Paragraph {
                children: vec![Node::Text(String::from("문단 내용"))],
            }],
        };

        match doc {
            Node::Document { children } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Paragraph { children } => {
                        assert_eq!(children.len(), 1);
                        match &children[0] {
                            Node::Text(s) => assert_eq!(s, "문단 내용"),
                            _ => panic!("Expected Text"),
                        }
                    }
                    _ => panic!("Expected Paragraph"),
                }
            }
            _ => panic!("Expected Document"),
        }
    }

    #[test]
    fn create_heading_node() {
        let heading = Node::Heading {
            level: 2,
            children: vec![Node::Text(String::from("제목"))],
        };

        match heading {
            Node::Heading { level, children } => {
                assert_eq!(level, 2);
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Text(s) => assert_eq!(s, "제목"),
                    _ => panic!("Expected Text"),
                }
            }
            _ => panic!("Expected Heading"),
        }
    }
}
