use crate::node::Node;

pub fn parse(input: &str) -> Node {
    if input.is_empty() {
        return Node::Document { children: vec![] };
    }

    let children = input.split("\n\n").filter(|s| !s.is_empty()).map(|block| {
        let block = block.trim();

        // Heading 검사: #로 시작하는지
        if block.starts_with('#') {
            // # 개수 세기
            let level = block.chars().take_while(|c| *c == '#').count();

            // 레벨 1~6만 유효, 7개 이상은 Paragraph
            if level >= 1 && level <= 6 {
                // # 뒤의 내용 추출 (앞뒤 공백 제거)
                let content = block[level..].trim();
                return Node::Heading {
                    level: level as u8,
                    children: vec![Node::Text(content.to_string())],
                };
            }
        }

        // 기본: Paragraph
        Node::Paragraph {
            children: vec![Node::Text(block.to_string())],
        }
    }).collect();

    Node::Document { children }
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

    #[test]
    fn parse_leading_blank_line() {
        // 앞에 빈 줄이 있는 경우 → 빈 줄은 무시되어야 함
        let doc = parse("\n\nparagraph");

        match doc {
            Node::Document { children } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Paragraph { children } => {
                        match &children[0] {
                            Node::Text(s) => assert_eq!(s, "paragraph"),
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
    fn parse_trailing_blank_line() {
        // 뒤에 빈 줄이 있는 경우 → 빈 줄은 무시되어야 함
        let doc = parse("paragraph\n\n");

        match doc {
            Node::Document { children } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Paragraph { children } => {
                        match &children[0] {
                            Node::Text(s) => assert_eq!(s, "paragraph"),
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
    fn parse_multiple_blank_lines() {
        // 연속 빈 줄(3개 이상의 개행)이 있는 경우
        let doc = parse("first\n\n\nsecond");

        match doc {
            Node::Document { children } => {
                assert_eq!(children.len(), 2);
                match &children[0] {
                    Node::Paragraph { children } => {
                        match &children[0] {
                            Node::Text(s) => assert_eq!(s, "first"),
                            _ => panic!("Expected Text"),
                        }
                    }
                    _ => panic!("Expected Paragraph"),
                }
                match &children[1] {
                    Node::Paragraph { children } => {
                        match &children[0] {
                            // 주목: 앞에 \n이 붙어있음
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

    #[test]
    fn parse_h1_heading() {
        let doc = parse("# heading");

        match doc {
            Node::Document { children } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Heading { level, children } => {
                        assert_eq!(*level, 1);
                        assert_eq!(children.len(), 1);
                        match &children[0] {
                            Node::Text(s) => assert_eq!(s, "heading"),
                            _ => panic!("Expected Text"),
                        }
                    }
                    _ => panic!("Expected Heading"),
                }
            }
            _ => panic!("Expected Document"),
        }
    }
}
