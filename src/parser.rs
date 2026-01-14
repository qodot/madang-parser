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
                let rest = &block[level..];

                // # 뒤에 공백/탭이 있거나 빈 제목이어야 Heading
                if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
                    let content = rest.trim();
                    return Node::Heading {
                        level: level as u8,
                        children: vec![Node::Text(content.to_string())],
                    };
                }
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
        assert_eq!(doc.children().len(), 0);
    }

    #[test]
    fn parse_simple_text() {
        let doc = parse("hello");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "hello");
    }

    #[test]
    fn parse_two_paragraphs() {
        let doc = parse("first\n\nsecond");

        assert_eq!(doc.children().len(), 2);
        assert_eq!(doc.children()[0].children()[0].as_text(), "first");
        assert_eq!(doc.children()[1].children()[0].as_text(), "second");
    }

    #[test]
    fn parse_leading_blank_line() {
        let doc = parse("\n\nparagraph");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "paragraph");
    }

    #[test]
    fn parse_trailing_blank_line() {
        let doc = parse("paragraph\n\n");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "paragraph");
    }

    #[test]
    fn parse_multiple_blank_lines() {
        let doc = parse("first\n\n\nsecond");

        assert_eq!(doc.children().len(), 2);
        assert_eq!(doc.children()[0].children()[0].as_text(), "first");
        assert_eq!(doc.children()[1].children()[0].as_text(), "second");
    }

    #[test]
    fn parse_h1_heading() {
        let doc = parse("# heading");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "heading");
    }

    #[test]
    fn parse_heading_requires_space() {
        let doc = parse("#no_space");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "#no_space");
    }

    #[test]
    fn parse_h6_heading() {
        let doc = parse("###### h6 title");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 6);
        assert_eq!(doc.children()[0].children()[0].as_text(), "h6 title");
    }

    #[test]
    fn parse_seven_hashes_is_paragraph() {
        let doc = parse("####### not heading");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "####### not heading");
    }
}
