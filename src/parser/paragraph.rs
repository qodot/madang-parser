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

    /// CommonMark Paragraph 예제 테스트
    #[rstest]
    // Example 219: 빈 줄로 분리된 두 Paragraph
    #[case("aaa\n\nbbb", &["aaa", "bbb"])]
    // Example 220: 여러 줄 Paragraph (soft line break)
    #[case("aaa\nbbb\n\nccc\nddd", &["aaa\nbbb", "ccc\nddd"])]
    // Example 221: 여러 빈 줄로 분리
    #[case("aaa\n\n\nbbb", &["aaa", "bbb"])]
    // Example 222: 선행 공백 제거
    #[case("  aaa\n bbb", &["aaa\nbbb"])]
    // Example 223: 들여쓰기된 continuation lines
    #[case("aaa\n         bbb\n                                       ccc", &["aaa\nbbb\nccc"])]
    // Example 224: 3칸 들여쓰기 허용
    #[case("   aaa\nbbb", &["aaa\nbbb"])]
    // 추가 케이스
    #[case("hello", &["hello"])]
    #[case("\n\nparagraph", &["paragraph"])]
    #[case("paragraph\n\n", &["paragraph"])]
    fn test_paragraph(#[case] input: &str, #[case] expected: &[&str]) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), expected.len(), "입력: {}", input);
        for (i, text) in expected.iter().enumerate() {
            assert_eq!(doc.children()[i].children()[0].as_text(), *text, "입력: {}", input);
        }
    }

    /// 복수 블록 조합 테스트 (code block + paragraph)
    #[rstest]
    // Example 225: 4칸 들여쓰기 → code block + paragraph
    #[case("    aaa\nbbb", 2)]
    fn test_block_sequence(#[case] input: &str, #[case] child_count: usize) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), child_count, "입력: {}", input);
        assert!(doc.children()[0].is_code_block(), "첫 번째는 CodeBlock, 입력: {}", input);
        assert!(!doc.children()[1].is_code_block(), "두 번째는 CodeBlock 아님, 입력: {}", input);
    }
}
