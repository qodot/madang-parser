//! Thematic Break 파싱
//!
//! CommonMark 명세: https://spec.commonmark.org/0.31.2/#thematic-breaks

use crate::node::Node;

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

#[cfg(test)]
mod tests {
    use crate::parser::parse;
    use rstest::rstest;

    // is_break = true면 ThematicBreak, false면 Paragraph
    #[rstest]
    // 기본 케이스 (3개)
    #[case("***", true)]
    #[case("---", true)]
    #[case("___", true)]
    // 3개 이상
    #[case("*****", true)]
    #[case("----------", true)]
    // 문자 사이 공백
    #[case("* * *", true)]
    #[case("- - -", true)]
    #[case("_  _  _", true)]                         // 여러 공백
    // 선행 공백 (0~3칸)
    #[case(" ***", true)]
    #[case("  ---", true)]
    #[case("   ___", true)]
    // 끝 공백
    #[case("***   ", true)]
    // Thematic Break가 아닌 케이스
    #[case("**", false)]                             // 2개 부족
    #[case("--", false)]
    #[case("__", false)]
    #[case("    ***", false)]                        // 4칸 들여쓰기
    #[case("*-*", false)]                            // 혼합 문자
    #[case("***a", false)]                           // 다른 문자 포함
    #[case("a]***", false)]                          // 앞에 다른 문자
    fn test_thematic_break(#[case] input: &str, #[case] is_break: bool) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "입력: {}", input);
        assert_eq!(doc.children()[0].is_thematic_break(), is_break, "입력: {}", input);
    }
}
