use crate::node::Node;

pub fn parse(input: &str) -> Node {
    if input.is_empty() {
        return Node::Document { children: vec![] };
    }

    let paragraphs = input.split("\n\n").map(|paragraph| {
        Node::Paragraph {
            children: vec![Node::Text(paragraph.to_string())],
        }
    }).collect();

    Node::Document { children: paragraphs }
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

    #[test]
    fn parse_two_paragraphs() {
        let doc = parse("first\n\nsecond");

        match doc {
            Node::Document { children } => {
                assert_eq!(children.len(), 2);
                // 첫 번째 문단
                match &children[0] {
                    Node::Paragraph { children } => {
                        match &children[0] {
                            Node::Text(s) => assert_eq!(s, "first"),
                            _ => panic!("Expected Text"),
                        }
                    }
                    _ => panic!("Expected Paragraph"),
                }
                // 두 번째 문단
                match &children[1] {
                    Node::Paragraph { children } => {
                        match &children[0] {
                            Node::Text(s) => assert_eq!(s, "second"),
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
