//! List Item 파서
//!
//! Bullet 마커 (-*+)와 Ordered 마커 (1. 1))를 감지합니다.

use super::context::{
    ListContinueReason, ListEndReason, ListItemNotStartReason, ListItemStart, ListItemStartReason,
    ListMarker,
};
use super::helpers::count_leading_char;

/// List Item 시작 줄인지 확인
/// 성공 시 Ok(Started), 실패 시 Err(사유) 반환
pub(crate) fn try_start(line: &str) -> Result<ListItemStartReason, ListItemNotStartReason> {
    let indent = count_leading_char(line, ' ');

    // 4칸 이상 들여쓰기는 코드 블록
    if indent > 3 {
        return Err(ListItemNotStartReason::CodeBlockIndented);
    }

    let after_indent = &line[indent..];

    // Bullet 또는 Ordered 마커 시도 → content 추출
    try_bullet_marker(after_indent, indent)
        .or_else(|| try_ordered_marker(after_indent, indent))
        .map(|start| ListItemStartReason::Started(start.with_content_from(line)))
        .ok_or(ListItemNotStartReason::NotListMarker)
}

/// List 종료 여부 확인
/// Ok: 종료 (Reprocess)
/// Err: 계속 (Blank, NewItem 또는 ContinuationLine)
pub(crate) fn try_end(
    line: &str,
    marker: &ListMarker,
    content_indent: usize,
) -> Result<ListEndReason, ListContinueReason> {
    // 빈 줄 처리 (항상 계속, 개수는 호출자가 추적)
    if line.trim().is_empty() {
        return Err(ListContinueReason::Blank);
    }

    // Continuation line: content_indent 이상 들여쓰기 확인
    // 중요: 새 아이템 체크보다 먼저! 중첩 리스트를 위해 필수.
    // 예: "- foo\n  - bar"에서 "  - bar"는 continuation line이어야 함
    let indent = count_leading_char(line, ' ');
    if indent >= content_indent {
        let content = line[content_indent..].to_string();
        return Err(ListContinueReason::ContinuationLine(content));
    }

    // 같은 마커 타입의 List Item이면 새 아이템으로 계속
    if let Ok(ListItemStartReason::Started(new_start)) = try_start(line) {
        if marker.is_same_type(&new_start.marker) {
            return Err(ListContinueReason::NewItem(new_start));
        }
    }

    // 다른 마커 또는 리스트가 아닌 내용 → 종료
    Ok(ListEndReason::Reprocess)
}

/// Bullet 마커 감지 (-*+)
fn try_bullet_marker(s: &str, indent: usize) -> Option<ListItemStart> {
    let first_char = s.chars().next()?;

    // Bullet 마커 문자인지 확인
    if !matches!(first_char, '-' | '+' | '*') {
        return None;
    }

    // 마커 뒤 공백 확인 (최소 1칸)
    let rest = &s[1..];
    if rest.is_empty() {
        // 마커만 있고 끝 → 빈 아이템으로 허용
        return Some(ListItemStart {
            marker: ListMarker::Bullet(first_char),
            indent,
            content_indent: indent + 1,
            content: String::new(), // try_start에서 채워짐
        });
    }

    // 마커 뒤 첫 문자가 공백이어야 함
    let after_marker = rest.chars().next()?;
    if after_marker != ' ' && after_marker != '\t' {
        return None;
    }

    // 내용 시작 위치 계산 (마커 + 공백)
    let spaces_after_marker = count_leading_char(rest, ' ');
    let content_indent = indent + 1 + spaces_after_marker.min(4); // 최대 4칸까지만

    Some(ListItemStart {
        marker: ListMarker::Bullet(first_char),
        indent,
        content_indent,
        content: String::new(), // try_start에서 채워짐
    })
}

/// Ordered 마커 감지 (숫자 + . 또는 ))
fn try_ordered_marker(s: &str, indent: usize) -> Option<ListItemStart> {
    // 숫자 추출
    let num_str: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();

    // 숫자가 없거나 9자리 초과면 실패
    if num_str.is_empty() || num_str.len() > 9 {
        return None;
    }

    // 0으로 시작하면 실패 (CommonMark 명세)
    if num_str.starts_with('0') {
        return None;
    }

    let start_num: usize = num_str.parse().ok()?;

    // 숫자 뒤 구분자 확인
    let rest = &s[num_str.len()..];
    let delimiter = rest.chars().next()?;

    if delimiter != '.' && delimiter != ')' {
        return None;
    }

    // 구분자 뒤 공백 확인
    let after_delimiter = &rest[1..];
    if after_delimiter.is_empty() {
        // 구분자만 있고 끝 → 빈 아이템
        let marker_len = num_str.len() + 1; // 숫자 + 구분자
        return Some(ListItemStart {
            marker: ListMarker::Ordered {
                start: start_num,
                delimiter,
            },
            indent,
            content_indent: indent + marker_len,
            content: String::new(), // try_start에서 채워짐
        });
    }

    // 구분자 뒤 첫 문자가 공백이어야 함
    let after_char = after_delimiter.chars().next()?;
    if after_char != ' ' && after_char != '\t' {
        return None;
    }

    // content_indent 계산
    let spaces_after_delimiter = count_leading_char(after_delimiter, ' ');
    let marker_len = num_str.len() + 1; // 숫자 + 구분자
    let content_indent = indent + marker_len + spaces_after_delimiter.min(4);

    Some(ListItemStart {
        marker: ListMarker::Ordered {
            start: start_num,
            delimiter,
        },
        indent,
        content_indent,
        content: String::new(), // try_start에서 채워짐
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::context::{ListContinueReason, ListEndReason};
    use rstest::rstest;

    /// try_start (Bullet) 테스트
    /// expected: Ok((marker_char, indent, content_indent)) 또는 Err(reason)
    #[rstest]
    // Example 261: 마커 뒤 공백 없으면 리스트 아님
    #[case("-item", Err(ListItemNotStartReason::NotListMarker))]
    #[case("--item", Err(ListItemNotStartReason::NotListMarker))]
    // 기본 Bullet 마커
    #[case("- item", Ok(('-', 0, 2)))]
    #[case("+ item", Ok(('+', 0, 2)))]
    #[case("* item", Ok(('*', 0, 2)))]
    // 마커 앞 들여쓰기 (0-3칸)
    #[case(" - item", Ok(('-', 1, 3)))]
    #[case("  - item", Ok(('-', 2, 4)))]
    #[case("   - item", Ok(('-', 3, 5)))]
    // 마커 뒤 여러 공백
    #[case("-  item", Ok(('-', 0, 3)))]
    #[case("-   item", Ok(('-', 0, 4)))]
    #[case("-    item", Ok(('-', 0, 5)))]
    #[case("-     item", Ok(('-', 0, 5)))]
    // 빈 아이템 (마커만)
    #[case("-", Ok(('-', 0, 1)))]
    #[case("+", Ok(('+', 0, 1)))]
    #[case("*", Ok(('*', 0, 1)))]
    // Bullet 마커가 아닌 경우
    #[case("    - item", Err(ListItemNotStartReason::CodeBlockIndented))]
    #[case("text", Err(ListItemNotStartReason::NotListMarker))]
    #[case("", Err(ListItemNotStartReason::NotListMarker))]
    fn test_bullet_marker(
        #[case] input: &str,
        #[case] expected: Result<(char, usize, usize), ListItemNotStartReason>,
    ) {
        let result = try_start(input);
        match expected {
            Ok((marker_char, indent, content_indent)) => {
                assert!(result.is_ok(), "Bullet이어야 함: {:?}", input);
                let ListItemStartReason::Started(start) = result.unwrap();
                assert_eq!(start.marker, ListMarker::Bullet(marker_char), "marker");
                assert_eq!(start.indent, indent, "indent");
                assert_eq!(start.content_indent, content_indent, "content_indent");
            }
            Err(expected_reason) => {
                assert!(result.is_err(), "Bullet이 아니어야 함: {:?}", input);
                assert_eq!(result.unwrap_err(), expected_reason);
            }
        }
    }

    /// try_start (Ordered) 테스트
    /// expected: Ok((start_num, delimiter, indent, content_indent)) 또는 Err(reason)
    #[rstest]
    // Example 261: 마커 뒤 공백 없으면 리스트 아님
    #[case("1.item", Err(ListItemNotStartReason::NotListMarker))]
    // 기본 Ordered 마커 (. 구분자)
    #[case("1. item", Ok((1, '.', 0, 3)))]
    #[case("2. item", Ok((2, '.', 0, 3)))]
    #[case("10. item", Ok((10, '.', 0, 4)))]
    #[case("123. item", Ok((123, '.', 0, 5)))]
    // 기본 Ordered 마커 () 구분자)
    #[case("1) item", Ok((1, ')', 0, 3)))]
    #[case("2) item", Ok((2, ')', 0, 3)))]
    #[case("10) item", Ok((10, ')', 0, 4)))]
    // 마커 앞 들여쓰기 (0-3칸)
    #[case(" 1. item", Ok((1, '.', 1, 4)))]
    #[case("  1. item", Ok((1, '.', 2, 5)))]
    #[case("   1. item", Ok((1, '.', 3, 6)))]
    // 마커 뒤 여러 공백
    #[case("1.  item", Ok((1, '.', 0, 4)))]
    #[case("1.   item", Ok((1, '.', 0, 5)))]
    // 9자리까지 허용
    #[case("123456789. item", Ok((123456789, '.', 0, 11)))]
    // 빈 아이템
    #[case("1.", Ok((1, '.', 0, 2)))]
    #[case("1)", Ok((1, ')', 0, 2)))]
    // Ordered 마커가 아닌 경우
    #[case("    1. item", Err(ListItemNotStartReason::CodeBlockIndented))]
    #[case("1234567890. item", Err(ListItemNotStartReason::NotListMarker))]
    #[case("0. item", Err(ListItemNotStartReason::NotListMarker))]
    #[case("a. item", Err(ListItemNotStartReason::NotListMarker))]
    #[case("1: item", Err(ListItemNotStartReason::NotListMarker))]
    fn test_ordered_marker(
        #[case] input: &str,
        #[case] expected: Result<(usize, char, usize, usize), ListItemNotStartReason>,
    ) {
        let result = try_start(input);
        match expected {
            Ok((start_num, delimiter, indent, content_indent)) => {
                assert!(result.is_ok(), "Ordered여야 함: {:?}", input);
                let ListItemStartReason::Started(start) = result.unwrap();
                match start.marker {
                    ListMarker::Ordered { start: s, delimiter: d } => {
                        assert_eq!(s, start_num, "start number");
                        assert_eq!(d, delimiter, "delimiter");
                    }
                    _ => panic!("Ordered 마커여야 함: {:?}", input),
                }
                assert_eq!(start.indent, indent, "indent");
                assert_eq!(start.content_indent, content_indent, "content_indent");
            }
            Err(expected_reason) => {
                // Ordered가 아니어야 함 (Bullet이거나 에러)
                match &result {
                    Ok(ListItemStartReason::Started(s)) => {
                        if let ListMarker::Ordered { .. } = s.marker {
                            panic!("Ordered가 아니어야 함: {:?}", input);
                        }
                    }
                    Err(reason) => {
                        assert_eq!(*reason, expected_reason);
                    }
                }
            }
        }
    }

    /// 빈 줄 처리 테스트 - 항상 Err(Blank) 반환
    #[rstest]
    #[case("")]
    #[case("  ")]
    #[case("\t")]
    fn test_try_end_blank_line(#[case] line: &str) {
        let marker = ListMarker::Bullet('-');
        let result = try_end(line, &marker, 2);
        assert!(matches!(result, Err(ListContinueReason::Blank)), "계속(Blank)이어야 함");
    }

    /// 같은 마커 타입 → 새 아이템으로 계속
    #[rstest]
    #[case("- b", ListMarker::Bullet('-'))]           // 같은 bullet
    #[case("- c", ListMarker::Bullet('-'))]
    #[case("+ b", ListMarker::Bullet('+'))]
    #[case("* b", ListMarker::Bullet('*'))]
    #[case("2. b", ListMarker::Ordered { start: 1, delimiter: '.' })]  // 같은 ordered (번호 달라도 OK)
    #[case("3. c", ListMarker::Ordered { start: 1, delimiter: '.' })]
    #[case("2) b", ListMarker::Ordered { start: 1, delimiter: ')' })]
    fn test_try_end_same_marker_continues(
        #[case] line: &str,
        #[case] marker: ListMarker,
    ) {
        let result = try_end(line, &marker, 2);
        assert!(matches!(result, Err(ListContinueReason::NewItem(_))), "새 아이템으로 계속해야 함: {:?}", result);
    }

    /// 리스트 종료 (Reprocess) 테스트: 다른 마커 또는 비리스트 내용
    #[rstest]
    // 다른 Bullet 마커
    #[case("+ b", ListMarker::Bullet('-'))]
    #[case("* b", ListMarker::Bullet('-'))]
    #[case("- b", ListMarker::Bullet('+'))]
    // 다른 Ordered delimiter
    #[case("1) b", ListMarker::Ordered { start: 1, delimiter: '.' })]
    #[case("1. b", ListMarker::Ordered { start: 1, delimiter: ')' })]
    // Ordered vs Bullet
    #[case("1. b", ListMarker::Bullet('-'))]
    #[case("- b", ListMarker::Ordered { start: 1, delimiter: '.' })]
    // 비리스트 내용
    #[case("some text", ListMarker::Bullet('-'))]
    #[case("# heading", ListMarker::Bullet('-'))]
    #[case("> blockquote", ListMarker::Bullet('-'))]
    #[case("```code", ListMarker::Bullet('-'))]
    fn test_try_end_reprocess(#[case] line: &str, #[case] marker: ListMarker) {
        let result = try_end(line, &marker, 2);
        assert!(
            matches!(result, Ok(ListEndReason::Reprocess)),
            "종료(Reprocess)여야 함: {:?}",
            result
        );
    }

    /// Continuation line (content_indent 이상 들여쓰기)
    #[rstest]
    #[case("  continued", 2, "continued")]     // 정확히 content_indent
    #[case("   continued", 2, " continued")]   // content_indent + 1
    #[case("    continued", 2, "  continued")] // content_indent + 2
    fn test_try_end_continuation_line(
        #[case] line: &str,
        #[case] content_indent: usize,
        #[case] expected_content: &str,
    ) {
        let marker = ListMarker::Bullet('-');
        let result = try_end(line, &marker, content_indent);
        match result {
            Err(ListContinueReason::ContinuationLine(content)) => {
                assert_eq!(content, expected_content, "continuation 내용");
            }
            _ => panic!("ContinuationLine이어야 함: {:?}", result),
        }
    }
}
