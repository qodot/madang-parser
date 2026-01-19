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
    use crate::node::Node;
    use crate::parser::parse;
    use rstest::rstest;

    #[rstest]
    // Example 219: 빈 줄로 분리된 두 Paragraph
    #[case("aaa\n\nbbb", vec![Node::para(vec![Node::text("aaa")]), Node::para(vec![Node::text("bbb")])])]
    // Example 220: 여러 줄 Paragraph (soft line break)
    #[case("aaa\nbbb\n\nccc\nddd", vec![Node::para(vec![Node::text("aaa\nbbb")]), Node::para(vec![Node::text("ccc\nddd")])])]
    // Example 221: 여러 빈 줄로 분리
    #[case("aaa\n\n\nbbb", vec![Node::para(vec![Node::text("aaa")]), Node::para(vec![Node::text("bbb")])])]
    // Example 222: 선행 공백 제거
    #[case("  aaa\n bbb", vec![Node::para(vec![Node::text("aaa\nbbb")])])]
    // Example 223: 들여쓰기된 continuation lines
    #[case("aaa\n         bbb\n                                       ccc", vec![Node::para(vec![Node::text("aaa\nbbb\nccc")])])]
    // Example 224: 3칸 들여쓰기 허용
    #[case("   aaa\nbbb", vec![Node::para(vec![Node::text("aaa\nbbb")])])]
    // Example 225: 4칸 들여쓰기 → code block + paragraph
    #[case("    aaa\nbbb", vec![Node::code_block(None, "aaa"), Node::para(vec![Node::text("bbb")])])]
    // 추가 케이스
    #[case("hello", vec![Node::para(vec![Node::text("hello")])])]
    #[case("\n\nparagraph", vec![Node::para(vec![Node::text("paragraph")])])]
    #[case("paragraph\n\n", vec![Node::para(vec![Node::text("paragraph")])])]
    fn test_paragraph(#[case] input: &str, #[case] expected: Vec<Node>) {
        let doc = parse(input);
        assert_eq!(doc.children(), &expected);
    }
}
