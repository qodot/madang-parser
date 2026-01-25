//! https://spec.commonmark.org/0.31.2/#thematic-breaks

/// Thematic Break 감지 성공 사유
#[derive(Debug, Clone, PartialEq)]
pub enum ThematicBreakOkReason {
    /// 유효한 Thematic Break 발견
    Started,
}

/// Thematic Break 감지 아님 사유
#[derive(Debug, Clone, PartialEq)]
pub enum ThematicBreakErrReason {
    /// 4칸 이상 들여쓰기 (코드 블록으로 해석됨)
    CodeBlockIndented,
    /// 빈 줄
    Empty,
    /// 유효하지 않은 마커 문자 (*, -, _ 아님)
    InvalidMarker,
    /// 마커 개수 부족 (3개 미만)
    InsufficientMarkers,
    /// 다른 문자 섞임 (공백/탭 외)
    MixedCharacters,
}

/// Thematic Break 파싱 시도
/// 성공 시 Ok(Started), 실패 시 Err(사유) 반환
pub fn parse(
    trimmed: &str,
    indent: usize,
) -> Result<ThematicBreakOkReason, ThematicBreakErrReason> {
    // 들여쓰기 3칸 초과면 코드 블록
    if indent > 3 {
        return Err(ThematicBreakErrReason::CodeBlockIndented);
    }

    // 빈 줄
    let trimmed = trimmed.trim();
    if trimmed.is_empty() {
        return Err(ThematicBreakErrReason::Empty);
    }

    // 첫 문자가 마커 문자인지 확인
    let first = trimmed.chars().next().unwrap();
    if first != '*' && first != '-' && first != '_' {
        return Err(ThematicBreakErrReason::InvalidMarker);
    }

    // 모든 문자가 같은 마커이거나 공백/탭인지 확인
    let mut marker_count = 0;
    for c in trimmed.chars() {
        if c == first {
            marker_count += 1;
        } else if c != ' ' && c != '\t' {
            return Err(ThematicBreakErrReason::MixedCharacters);
        }
    }

    // 마커가 3개 이상이어야 함
    if marker_count < 3 {
        return Err(ThematicBreakErrReason::InsufficientMarkers);
    }

    Ok(ThematicBreakOkReason::Started)
}

#[cfg(test)]
mod tests {
    use crate::node::Node;
    use crate::parser::parse;
    use rstest::rstest;

    #[rstest]
    // Example 43: 기본 케이스
    #[case("***", vec![Node::ThematicBreak])]
    #[case("---", vec![Node::ThematicBreak])]
    #[case("___", vec![Node::ThematicBreak])]
    // Example 44-45: 유효하지 않은 마커 문자 → Paragraph
    #[case("+++", vec![Node::para(vec![Node::text("+++")])])]
    #[case("===", vec![Node::para(vec![Node::text("===")])])]
    // Example 46: 2개는 부족 → Paragraph
    #[case("**", vec![Node::para(vec![Node::text("**")])])]
    #[case("--", vec![Node::para(vec![Node::text("--")])])]
    #[case("__", vec![Node::para(vec![Node::text("__")])])]
    // Example 47: 1-3칸 들여쓰기 허용
    #[case(" ***", vec![Node::ThematicBreak])]
    #[case("  ***", vec![Node::ThematicBreak])]
    #[case("   ***", vec![Node::ThematicBreak])]
    // Example 48: 4칸 들여쓰기는 코드 블록
    #[case("    ***", vec![Node::code_block(None, "***")])]
    // Example 49: Paragraph 내 4칸 들여쓰기는 continuation
    #[case("Foo\n    ***", vec![Node::para(vec![Node::text("Foo\n***")])])]
    // Example 50: 많은 문자
    #[case("_____________________________________", vec![Node::ThematicBreak])]
    // Example 51: 공백 사이
    #[case(" - - -", vec![Node::ThematicBreak])]
    // Example 52: 복잡한 공백 패턴
    #[case(" **  * ** * ** * **", vec![Node::ThematicBreak])]
    // Example 53: 많은 공백
    #[case("-     -      -      -", vec![Node::ThematicBreak])]
    // Example 54: 끝 공백
    #[case("- - - -    ", vec![Node::ThematicBreak])]
    // Example 55: 다른 문자 포함 시 무효 → Paragraph
    #[case("_ _ _ _ a", vec![Node::para(vec![Node::text("_ _ _ _ a")])])]
    #[case("a------", vec![Node::para(vec![Node::text("a------")])])]
    #[case("---a---", vec![Node::para(vec![Node::text("---a---")])])]
    // Example 56: 혼합 문자는 무효 → Paragraph
    #[case("*-*", vec![Node::para(vec![Node::text("*-*")])])]
    // 추가 케이스
    #[case("*****", vec![Node::ThematicBreak])]
    #[case("----------", vec![Node::ThematicBreak])]
    #[case("* * *", vec![Node::ThematicBreak])]
    #[case("- - -", vec![Node::ThematicBreak])]
    #[case("_  _  _", vec![Node::ThematicBreak])]
    #[case("  ---", vec![Node::ThematicBreak])]
    #[case("   ___", vec![Node::ThematicBreak])]
    #[case("***   ", vec![Node::ThematicBreak])]
    #[case("***a", vec![Node::para(vec![Node::text("***a")])])]
    fn test_thematic_break(#[case] input: &str, #[case] expected: Vec<Node>) {
        let doc = parse(input);
        assert_eq!(doc.children(), &expected);
    }
}
