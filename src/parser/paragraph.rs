//! Paragraph 파싱
//!
//! CommonMark 명세: https://spec.commonmark.org/0.31.2/#paragraphs

use crate::node::Node;

/// Paragraph 파싱 (기본 fallback)
/// 다른 블록 요소가 아닌 경우 항상 Paragraph로 처리
pub fn parse(trimmed: &str) -> Node {
    Node::Paragraph {
        children: vec![Node::Text(trimmed.to_string())],
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;
    use rstest::rstest;

    #[rstest]
    #[case("hello", &["hello"])]
    #[case("\n\nparagraph", &["paragraph"])]           // 앞 빈 줄
    #[case("paragraph\n\n", &["paragraph"])]           // 뒤 빈 줄
    #[case("first\n\nsecond", &["first", "second"])]
    #[case("first\n\n\nsecond", &["first", "second"])] // 연속 빈 줄
    fn test_paragraph(#[case] input: &str, #[case] expected: &[&str]) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), expected.len(), "입력: {}", input);
        for (i, text) in expected.iter().enumerate() {
            assert_eq!(doc.children()[i].children()[0].as_text(), *text, "입력: {}", input);
        }
    }
}
