//! Setext Heading 파서
//!
//! 밑줄 스타일의 제목을 처리합니다.
//! - `=` 밑줄: 레벨 1
//! - `-` 밑줄: 레벨 2

use super::context::{
    HeadingSetextNotStartReason, HeadingSetextStart, HeadingSetextStartReason, SetextLevel,
};

/// Setext 밑줄인지 확인하고 레벨 반환
///
/// - `Ok(HeadingSetextStartReason::Started(start))`: 유효한 밑줄
/// - `Err(reason)`: 유효하지 않음
pub fn try_start(line: &str, indent: usize) -> Result<HeadingSetextStartReason, HeadingSetextNotStartReason> {
    // 들여쓰기 4칸 이상이면 코드 블록
    if indent > 3 {
        return Err(HeadingSetextNotStartReason::CodeBlockIndented);
    }

    // 후행 공백 제거
    let trimmed = line.trim_end();

    // 빈 문자열은 밑줄이 아님
    if trimmed.is_empty() {
        return Err(HeadingSetextNotStartReason::Empty);
    }

    // 첫 문자로 레벨 결정
    let first = trimmed.chars().next().unwrap();
    let level = match first {
        '=' => SetextLevel::Level1,
        '-' => SetextLevel::Level2,
        _ => return Err(HeadingSetextNotStartReason::NotUnderlineChar),
    };

    // 모든 문자가 같은지 확인
    if trimmed.chars().all(|c| c == first) {
        Ok(HeadingSetextStartReason::Started(HeadingSetextStart { level }))
    } else {
        Err(HeadingSetextNotStartReason::MixedChars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// try_start 테스트: 유효/무효 케이스 통합
    #[rstest]
    // === 유효한 밑줄: 레벨 1 (=) ===
    #[case("=", 0, Ok(SetextLevel::Level1))]
    #[case("==", 0, Ok(SetextLevel::Level1))]
    #[case("===", 0, Ok(SetextLevel::Level1))]
    #[case("==========", 0, Ok(SetextLevel::Level1))]
    #[case("===   ", 0, Ok(SetextLevel::Level1))]  // 후행 공백 허용
    // === 유효한 밑줄: 레벨 2 (-) ===
    #[case("-", 0, Ok(SetextLevel::Level2))]
    #[case("--", 0, Ok(SetextLevel::Level2))]
    #[case("---", 0, Ok(SetextLevel::Level2))]
    #[case("----------", 0, Ok(SetextLevel::Level2))]
    #[case("---   ", 0, Ok(SetextLevel::Level2))]  // 후행 공백 허용
    // === 들여쓰기: 0-3칸 허용 ===
    #[case("===", 1, Ok(SetextLevel::Level1))]
    #[case("===", 2, Ok(SetextLevel::Level1))]
    #[case("===", 3, Ok(SetextLevel::Level1))]
    // === 무효: 4칸 이상 들여쓰기 → CodeBlockIndented ===
    #[case("===", 4, Err(HeadingSetextNotStartReason::CodeBlockIndented))]
    #[case("===", 5, Err(HeadingSetextNotStartReason::CodeBlockIndented))]
    // === 무효: 빈 줄 → Empty ===
    #[case("", 0, Err(HeadingSetextNotStartReason::Empty))]
    #[case("   ", 0, Err(HeadingSetextNotStartReason::Empty))]
    // === 무효: 밑줄 문자가 아님 → NotUnderlineChar ===
    #[case("abc", 0, Err(HeadingSetextNotStartReason::NotUnderlineChar))]
    #[case("###", 0, Err(HeadingSetextNotStartReason::NotUnderlineChar))]
    // === 무효: 문자 섞임 → MixedChars ===
    #[case("= =", 0, Err(HeadingSetextNotStartReason::MixedChars))]
    #[case("- -", 0, Err(HeadingSetextNotStartReason::MixedChars))]
    #[case("== ==", 0, Err(HeadingSetextNotStartReason::MixedChars))]
    #[case("=-=", 0, Err(HeadingSetextNotStartReason::MixedChars))]
    #[case("==-", 0, Err(HeadingSetextNotStartReason::MixedChars))]
    // === 무효: 밑줄 뒤 비공백 문자 → MixedChars ===
    #[case("=== bar", 0, Err(HeadingSetextNotStartReason::MixedChars))]
    #[case("--- foo", 0, Err(HeadingSetextNotStartReason::MixedChars))]
    fn test_try_start(
        #[case] line: &str,
        #[case] indent: usize,
        #[case] expected: Result<SetextLevel, HeadingSetextNotStartReason>,
    ) {
        let result = try_start(line, indent);
        match expected {
            Ok(level) => assert_eq!(
                result,
                Ok(HeadingSetextStartReason::Started(HeadingSetextStart { level }))
            ),
            Err(reason) => assert_eq!(result, Err(reason)),
        }
    }

    /// Setext Heading 통합 테스트
    #[rstest]
    // 레벨 1 (=)
    #[case("Foo\n===", 1, "Foo")]
    #[case("Foo\n=", 1, "Foo")]
    #[case("Foo\n==========", 1, "Foo")]
    // 레벨 2 (-)
    #[case("Foo\n---", 2, "Foo")]
    #[case("Foo\n-", 2, "Foo")]
    #[case("Foo\n----------", 2, "Foo")]
    // 여러 줄 제목
    #[case("Foo\nbar\n===", 1, "Foo\nbar")]
    #[case("Foo\nbar\nbaz\n---", 2, "Foo\nbar\nbaz")]
    // 밑줄에 후행 공백
    #[case("Foo\n===   ", 1, "Foo")]
    #[case("Foo\n---   ", 2, "Foo")]
    // 밑줄에 선행 들여쓰기 (1-3칸 허용)
    #[case("Foo\n ===", 1, "Foo")]
    #[case("Foo\n  ===", 1, "Foo")]
    #[case("Foo\n   ===", 1, "Foo")]
    #[case("Foo\n ---", 2, "Foo")]
    // 제목 텍스트에 들여쓰기 (1-3칸 허용)
    #[case(" Foo\n===", 1, "Foo")]
    #[case("  Foo\n===", 1, "Foo")]
    #[case("   Foo\n===", 1, "Foo")]
    fn test_setext_heading(#[case] input: &str, #[case] level: u8, #[case] content: &str) {
        let doc = crate::parse(input);
        assert_eq!(doc.children().len(), 1, "입력: {:?}", input);
        let heading = &doc.children()[0];
        assert!(heading.is_heading(), "Heading이 아님: {:?}", heading);
        assert_eq!(heading.level(), level, "레벨 불일치: {:?}", input);
        assert_eq!(heading.children()[0].as_text(), content, "내용 불일치: {:?}", input);
    }

    /// Setext Heading이 아닌 케이스 테스트
    #[rstest]
    // 밑줄에 4칸 이상 들여쓰기 → Setext 아님, Paragraph continuation
    #[case("Foo\n    ===", 1)]  // Paragraph("Foo\n===")
    // 빈 줄 후 밑줄 → Setext 아님
    #[case("Foo\n\n===", 2)]  // Paragraph("Foo"), Paragraph("===")
    // 밑줄만 단독 (=) → Paragraph
    #[case("===", 1)]
    // 밑줄만 단독 (-) → Thematic Break
    #[case("---", 1)]
    // 밑줄 뒤 비공백 문자 → Setext 아님, Paragraph continuation
    #[case("Foo\n=== bar", 1)]  // Paragraph("Foo\n=== bar")
    fn test_not_setext_heading(#[case] input: &str, #[case] expected_children: usize) {
        let doc = crate::parse(input);
        assert_eq!(
            doc.children().len(),
            expected_children,
            "자식 개수 불일치. 입력: {:?}, 결과: {:?}",
            input,
            doc
        );
        // 첫 번째 자식이 Heading이 아님을 확인
        let first = &doc.children()[0];
        assert!(
            !first.is_heading(),
            "Heading이면 안됨. 입력: {:?}, 결과: {:?}",
            input,
            first
        );
    }
}
