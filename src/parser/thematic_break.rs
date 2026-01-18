//! Thematic Break 파싱
//!
//! CommonMark 명세: https://spec.commonmark.org/0.31.2/#thematic-breaks

use crate::node::Node;

/// Thematic Break 파싱 시도
/// 성공하면 Some(Node::ThematicBreak), 실패하면 None
pub fn parse(trimmed: &str, indent: usize) -> Option<Node> {
    // 들여쓰기 3칸 초과면 Thematic Break 아님
    if indent > 3 {
        return None;
    }

    if is_thematic_break(trimmed) {
        Some(Node::ThematicBreak)
    } else {
        None
    }
}

/// Thematic Break 검사
/// 규칙: *, -, _ 중 하나가 3개 이상, 공백/탭만 사이에 허용
fn is_thematic_break(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }

    // 첫 문자가 마커 문자인지 확인
    let first = trimmed.chars().next().unwrap();
    if first != '*' && first != '-' && first != '_' {
        return false;
    }

    // 모든 문자가 같은 마커이거나 공백/탭인지 확인
    let mut marker_count = 0;
    for c in trimmed.chars() {
        if c == first {
            marker_count += 1;
        } else if c != ' ' && c != '\t' {
            return false;
        }
    }

    // 마커가 3개 이상이어야 함
    marker_count >= 3
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;
    use rstest::rstest;

    /// Thematic Break 테스트
    /// is_break = true면 ThematicBreak, false면 Paragraph
    #[rstest]
    // Example 43: 기본 케이스
    #[case("***", true)]
    #[case("---", true)]
    #[case("___", true)]
    // Example 44-45: 유효하지 않은 마커 문자
    #[case("+++", false)]
    #[case("===", false)]
    // Example 46: 2개는 부족
    #[case("**", false)]
    #[case("--", false)]
    #[case("__", false)]
    // Example 47: 1-3칸 들여쓰기 허용
    #[case(" ***", true)]
    #[case("  ***", true)]
    #[case("   ***", true)]
    // Example 48: 4칸 들여쓰기는 코드 블록
    #[case("    ***", false)]
    // Example 50: 많은 문자
    #[case("_____________________________________", true)]
    // Example 51: 공백 사이
    #[case(" - - -", true)]
    // Example 52: 복잡한 공백 패턴
    #[case(" **  * ** * ** * **", true)]
    // Example 53: 많은 공백
    #[case("-     -      -      -", true)]
    // Example 54: 끝 공백
    #[case("- - - -    ", true)]
    // Example 55: 다른 문자 포함 시 무효
    #[case("_ _ _ _ a", false)]
    #[case("a------", false)]
    #[case("---a---", false)]
    // Example 56: 혼합 문자는 무효
    #[case("*-*", false)]
    // 추가 케이스
    #[case("*****", true)]
    #[case("----------", true)]
    #[case("* * *", true)]
    #[case("- - -", true)]
    #[case("_  _  _", true)]
    #[case("  ---", true)]
    #[case("   ___", true)]
    #[case("***   ", true)]
    #[case("***a", false)]
    fn test_thematic_break(#[case] input: &str, #[case] is_break: bool) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "입력: {}", input);
        assert_eq!(doc.children()[0].is_thematic_break(), is_break, "입력: {}", input);
    }
}
