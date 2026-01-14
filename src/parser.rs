use crate::node::Node;

pub fn parse(input: &str) -> Node {
    if input.is_empty() {
        return Node::Document { children: vec![] };
    }

    Node::Document {
        children: vec![Node::Paragraph {
            children: vec![Node::Text(input.to_string())],
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_string() {
        let doc = parse("");

        match doc {
            Node::Document { children } => {
                assert_eq!(children.len(), 0);
            }
            _ => panic!("Expected Document"),
        }
    }

    #[test]
    fn parse_simple_text() {
        let doc = parse("hello");

        match doc {
            Node::Document { children } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Paragraph { children } => {
                        assert_eq!(children.len(), 1);
                        match &children[0] {
                            Node::Text(s) => assert_eq!(s, "hello"),
                            _ => panic!("Expected Text"),
                        }
                    }
                    _ => panic!("Expected Paragraph"),
                }
            }
            _ => panic!("Expected Document"),
        }
    }
}
