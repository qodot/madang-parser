//! List Item 파서
//!
//! Bullet 마커 (-*+)와 Ordered 마커 (1. 1))를 감지합니다.

use super::context::{ListContinueReason, ListEndReason, ListItemStart, ListMarker};
use super::helpers::count_leading_char;

/// List Item 시작 줄인지 확인
/// 성공 시 시작 정보(ListItemStart) 반환
pub(crate) fn try_start(line: &str) -> Option<ListItemStart> {
    let indent = count_leading_char(line, ' ');

    // 4칸 이상 들여쓰기는 코드 블록
    if indent > 3 {
        return None;
    }

    let after_indent = &line[indent..];

    // Bullet 또는 Ordered 마커 시도 → content 추출
    try_bullet_marker(after_indent, indent)
        .or_else(|| try_ordered_marker(after_indent, indent))
        .map(|start| start.with_content_from(line))
}

/// List 종료 여부 확인
/// Ok: 종료 (Reprocess 또는 Consumed)
/// Err: 계속 (Blank 또는 NewItem)
pub(crate) fn try_end(
    line: &str,
    marker: &ListMarker,
    pending_blank: bool,
) -> Result<ListEndReason, ListContinueReason> {
    // 빈 줄 처리
    if line.trim().is_empty() {
        return if pending_blank {
            Ok(ListEndReason::Consumed) // 두 번 연속 빈 줄 → 종료
        } else {
            Err(ListContinueReason::Blank) // 첫 번째 빈 줄 → 계속
        };
    }

    // 같은 마커 타입의 List Item이면 계속
    if let Some(new_start) = try_start(line) {
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

    // === try_start (Bullet) 테스트 ===
    // expected: Some((marker_char, indent, content_indent)) 또는 None
    #[rstest]
    // 기본 Bullet 마커
    #[case("- item", Some(('-', 0, 2)))]
    #[case("+ item", Some(('+', 0, 2)))]
    #[case("* item", Some(('*', 0, 2)))]
    // 마커 앞 들여쓰기 (0-3칸)
    #[case(" - item", Some(('-', 1, 3)))]
    #[case("  - item", Some(('-', 2, 4)))]
    #[case("   - item", Some(('-', 3, 5)))]
    // 마커 뒤 여러 공백
    #[case("-  item", Some(('-', 0, 3)))]
    #[case("-   item", Some(('-', 0, 4)))]
    #[case("-    item", Some(('-', 0, 5)))]   // 최대 4칸
    #[case("-     item", Some(('-', 0, 5)))]  // 5칸이어도 content_indent는 5
    // 빈 아이템 (마커만)
    #[case("-", Some(('-', 0, 1)))]
    #[case("+", Some(('+', 0, 1)))]
    #[case("*", Some(('*', 0, 1)))]
    // Bullet 마커가 아닌 경우
    #[case("    - item", None)]  // 4칸 들여쓰기 → 코드 블록
    #[case("-item", None)]       // 마커 뒤 공백 없음
    #[case("--item", None)]      // 두 번째 문자도 마커
    #[case("text", None)]        // 일반 텍스트
    #[case("", None)]            // 빈 줄
    fn test_bullet_marker(
        #[case] input: &str,
        #[case] expected: Option<(char, usize, usize)>,
    ) {
        let result = try_start(input);
        match expected {
            Some((marker_char, indent, content_indent)) => {
                assert!(result.is_some(), "Bullet이어야 함: {:?}", input);
                let start = result.unwrap();
                assert_eq!(start.marker, ListMarker::Bullet(marker_char), "marker");
                assert_eq!(start.indent, indent, "indent");
                assert_eq!(start.content_indent, content_indent, "content_indent");
            }
            None => {
                assert!(result.is_none(), "Bullet이 아니어야 함: {:?}", input);
            }
        }
    }

    // === try_start (Ordered) 테스트 ===
    // expected: Some((start_num, delimiter, indent, content_indent)) 또는 None
    #[rstest]
    // 기본 Ordered 마커 (. 구분자)
    #[case("1. item", Some((1, '.', 0, 3)))]
    #[case("2. item", Some((2, '.', 0, 3)))]
    #[case("10. item", Some((10, '.', 0, 4)))]
    #[case("123. item", Some((123, '.', 0, 5)))]
    // 기본 Ordered 마커 () 구분자)
    #[case("1) item", Some((1, ')', 0, 3)))]
    #[case("2) item", Some((2, ')', 0, 3)))]
    #[case("10) item", Some((10, ')', 0, 4)))]
    // 마커 앞 들여쓰기 (0-3칸)
    #[case(" 1. item", Some((1, '.', 1, 4)))]
    #[case("  1. item", Some((1, '.', 2, 5)))]
    #[case("   1. item", Some((1, '.', 3, 6)))]
    // 마커 뒤 여러 공백
    #[case("1.  item", Some((1, '.', 0, 4)))]
    #[case("1.   item", Some((1, '.', 0, 5)))]
    // 9자리까지 허용
    #[case("123456789. item", Some((123456789, '.', 0, 11)))]
    // 빈 아이템
    #[case("1.", Some((1, '.', 0, 2)))]
    #[case("1)", Some((1, ')', 0, 2)))]
    // Ordered 마커가 아닌 경우
    #[case("    1. item", None)]      // 4칸 들여쓰기 → 코드 블록
    #[case("1.item", None)]           // 마커 뒤 공백 없음
    #[case("1234567890. item", None)] // 10자리 → 너무 김
    #[case("0. item", None)]          // 0으로 시작 (선택: 허용할지 말지)
    #[case("a. item", None)]          // 문자
    #[case("1: item", None)]          // 잘못된 구분자
    fn test_ordered_marker(
        #[case] input: &str,
        #[case] expected: Option<(usize, char, usize, usize)>,
    ) {
        let result = try_start(input);
        match expected {
            Some((start_num, delimiter, indent, content_indent)) => {
                assert!(result.is_some(), "Ordered여야 함: {:?}", input);
                let start = result.unwrap();
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
            None => {
                // Ordered가 아니어야 함 (Bullet이거나 None)
                if let Some(s) = &result {
                    if let ListMarker::Ordered { .. } = s.marker {
                        panic!("Ordered가 아니어야 함: {:?}", input);
                    }
                }
            }
        }
    }

    // === try_end 테스트 ===

    /// 빈 줄 처리 테스트
    /// pending_blank=false: 첫 빈 줄 → Err(Blank)
    /// pending_blank=true: 두 번째 빈 줄 → Ok(Consumed)
    #[rstest]
    #[case("", false, false)]   // 빈 줄, pending=false → 계속 (Blank)
    #[case("  ", false, false)] // 공백만, pending=false → 계속 (Blank)
    #[case("", true, true)]     // 빈 줄, pending=true → 종료 (Consumed)
    #[case("  ", true, true)]   // 공백만, pending=true → 종료 (Consumed)
    fn test_try_end_blank_line(
        #[case] line: &str,
        #[case] pending_blank: bool,
        #[case] should_end: bool,
    ) {
        let marker = ListMarker::Bullet('-');
        let result = try_end(line, &marker, pending_blank);

        if should_end {
            assert!(matches!(result, Ok(ListEndReason::Consumed)), "종료(Consumed)여야 함");
        } else {
            assert!(matches!(result, Err(ListContinueReason::Blank)), "계속(Blank)이어야 함");
        }
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
        let result = try_end(line, &marker, false);
        assert!(matches!(result, Err(ListContinueReason::NewItem(_))), "새 아이템으로 계속해야 함: {:?}", result);
    }

    /// 다른 마커 타입 → 종료 (Reprocess)
    #[rstest]
    #[case("+ b", ListMarker::Bullet('-'))]           // 다른 bullet
    #[case("* b", ListMarker::Bullet('-'))]
    #[case("- b", ListMarker::Bullet('+'))]
    #[case("1) b", ListMarker::Ordered { start: 1, delimiter: '.' })]  // 다른 delimiter
    #[case("1. b", ListMarker::Ordered { start: 1, delimiter: ')' })]
    #[case("1. b", ListMarker::Bullet('-'))]          // ordered vs bullet
    #[case("- b", ListMarker::Ordered { start: 1, delimiter: '.' })]   // bullet vs ordered
    fn test_try_end_different_marker_ends(
        #[case] line: &str,
        #[case] marker: ListMarker,
    ) {
        let result = try_end(line, &marker, false);
        assert!(matches!(result, Ok(ListEndReason::Reprocess)), "종료(Reprocess)여야 함: {:?}", result);
    }

    /// 리스트가 아닌 내용 → 종료 (Reprocess)
    #[rstest]
    #[case("some text")]
    #[case("# heading")]
    #[case("> blockquote")]
    #[case("```code")]
    fn test_try_end_non_list_content_ends(#[case] line: &str) {
        let marker = ListMarker::Bullet('-');
        let result = try_end(line, &marker, false);
        assert!(matches!(result, Ok(ListEndReason::Reprocess)), "종료(Reprocess)여야 함: {:?}", result);
    }
}
