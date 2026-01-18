//! List Item 파서 (CommonMark 5.2 List Items)
//!
//! Bullet 마커 (-*+)와 Ordered 마커 (1. 1))를 감지합니다.
//! - 마커 인식 규칙 (Example 261, 265-269)
//! - 들여쓰기 규칙
//! - Continuation line 판별

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
    use rstest::rstest;

    // 5.2 List Items - 마커 인식 (try_start)
    #[rstest]
    // Example 261: 마커 뒤 공백 필수
    #[case("-item", Err(ListItemNotStartReason::NotListMarker))]
    #[case("--item", Err(ListItemNotStartReason::NotListMarker))]
    #[case("1.item", Err(ListItemNotStartReason::NotListMarker))]
    // 기본 Bullet 마커
    #[case(
        "- item",
        Ok(ListItemStartReason::Started(ListItemStart::bullet('-', 0, 2, "item")))
    )]
    #[case(
        "+ item",
        Ok(ListItemStartReason::Started(ListItemStart::bullet('+', 0, 2, "item")))
    )]
    #[case(
        "* item",
        Ok(ListItemStartReason::Started(ListItemStart::bullet('*', 0, 2, "item")))
    )]
    // Bullet 마커 앞 들여쓰기 (0-3칸)
    #[case(
        " - item",
        Ok(ListItemStartReason::Started(ListItemStart::bullet('-', 1, 3, "item")))
    )]
    #[case(
        "  - item",
        Ok(ListItemStartReason::Started(ListItemStart::bullet('-', 2, 4, "item")))
    )]
    #[case(
        "   - item",
        Ok(ListItemStartReason::Started(ListItemStart::bullet('-', 3, 5, "item")))
    )]
    // Bullet 마커 뒤 여러 공백
    #[case(
        "-  item",
        Ok(ListItemStartReason::Started(ListItemStart::bullet('-', 0, 3, "item")))
    )]
    #[case(
        "-   item",
        Ok(ListItemStartReason::Started(ListItemStart::bullet('-', 0, 4, "item")))
    )]
    #[case(
        "-    item",
        Ok(ListItemStartReason::Started(ListItemStart::bullet('-', 0, 5, "item")))
    )]
    #[case(
        "-     item",
        Ok(ListItemStartReason::Started(ListItemStart::bullet('-', 0, 5, " item")))
    )]
    // Bullet 빈 아이템
    #[case(
        "-",
        Ok(ListItemStartReason::Started(ListItemStart::bullet('-', 0, 1, "")))
    )]
    #[case(
        "+",
        Ok(ListItemStartReason::Started(ListItemStart::bullet('+', 0, 1, "")))
    )]
    #[case(
        "*",
        Ok(ListItemStartReason::Started(ListItemStart::bullet('*', 0, 1, "")))
    )]
    // 기본 Ordered 마커
    #[case(
        "1. item",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(1, '.', 0, 3, "item")))
    )]
    #[case(
        "2. item",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(2, '.', 0, 3, "item")))
    )]
    #[case(
        "10. item",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(10, '.', 0, 4, "item")))
    )]
    #[case(
        "123. item",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(123, '.', 0, 5, "item")))
    )]
    #[case(
        "1) item",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(1, ')', 0, 3, "item")))
    )]
    #[case(
        "2) item",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(2, ')', 0, 3, "item")))
    )]
    #[case(
        "10) item",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(10, ')', 0, 4, "item")))
    )]
    // Ordered 마커 앞 들여쓰기
    #[case(
        " 1. item",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(1, '.', 1, 4, "item")))
    )]
    #[case(
        "  1. item",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(1, '.', 2, 5, "item")))
    )]
    #[case(
        "   1. item",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(1, '.', 3, 6, "item")))
    )]
    // Ordered 마커 뒤 여러 공백
    #[case(
        "1.  item",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(1, '.', 0, 4, "item")))
    )]
    #[case(
        "1.   item",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(1, '.', 0, 5, "item")))
    )]
    // 9자리까지 허용
    #[case(
        "123456789. item",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(123456789, '.', 0, 11, "item")))
    )]
    // Ordered 빈 아이템
    #[case(
        "1.",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(1, '.', 0, 2, "")))
    )]
    #[case(
        "1)",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(1, ')', 0, 2, "")))
    )]
    // 에러 케이스
    #[case("    - item", Err(ListItemNotStartReason::CodeBlockIndented))]
    #[case("    1. item", Err(ListItemNotStartReason::CodeBlockIndented))]
    #[case("text", Err(ListItemNotStartReason::NotListMarker))]
    #[case("", Err(ListItemNotStartReason::NotListMarker))]
    #[case("1234567890. item", Err(ListItemNotStartReason::NotListMarker))]
    #[case("0. item", Err(ListItemNotStartReason::NotListMarker))]
    #[case("a. item", Err(ListItemNotStartReason::NotListMarker))]
    #[case("1: item", Err(ListItemNotStartReason::NotListMarker))]
    fn test_try_start(
        #[case] input: &str,
        #[case] expected: Result<ListItemStartReason, ListItemNotStartReason>,
    ) {
        assert_eq!(try_start(input), expected);
    }

    // 5.2 List Items - 종료/계속 판별 (try_end)
    #[rstest]
    // 빈 줄 → Blank
    #[case("", ListMarker::Bullet('-'), 2, Err(ListContinueReason::Blank))]
    #[case("  ", ListMarker::Bullet('-'), 2, Err(ListContinueReason::Blank))]
    #[case("\t", ListMarker::Bullet('-'), 2, Err(ListContinueReason::Blank))]
    // 같은 마커 → 새 아이템
    #[case(
        "- b",
        ListMarker::Bullet('-'),
        2,
        Err(ListContinueReason::NewItem(ListItemStart::bullet('-', 0, 2, "b")))
    )]
    #[case(
        "+ b",
        ListMarker::Bullet('+'),
        2,
        Err(ListContinueReason::NewItem(ListItemStart::bullet('+', 0, 2, "b")))
    )]
    #[case(
        "* b",
        ListMarker::Bullet('*'),
        2,
        Err(ListContinueReason::NewItem(ListItemStart::bullet('*', 0, 2, "b")))
    )]
    #[case("2. b", ListMarker::Ordered { start: 1, delimiter: '.' }, 2, Err(ListContinueReason::NewItem(ListItemStart::ordered(2, '.', 0, 3, "b"))))]
    #[case("2) b", ListMarker::Ordered { start: 1, delimiter: ')' }, 2, Err(ListContinueReason::NewItem(ListItemStart::ordered(2, ')', 0, 3, "b"))))]
    // Continuation line
    #[case("  continued", ListMarker::Bullet('-'), 2, Err(ListContinueReason::ContinuationLine("continued".to_string())))]
    #[case("   continued", ListMarker::Bullet('-'), 2, Err(ListContinueReason::ContinuationLine(" continued".to_string())))]
    #[case("    continued", ListMarker::Bullet('-'), 2, Err(ListContinueReason::ContinuationLine("  continued".to_string())))]
    // 리스트 종료 (다른 마커)
    #[case("+ b", ListMarker::Bullet('-'), 2, Ok(ListEndReason::Reprocess))]
    #[case("* b", ListMarker::Bullet('-'), 2, Ok(ListEndReason::Reprocess))]
    #[case("- b", ListMarker::Bullet('+'), 2, Ok(ListEndReason::Reprocess))]
    #[case("1) b", ListMarker::Ordered { start: 1, delimiter: '.' }, 2, Ok(ListEndReason::Reprocess))]
    #[case("1. b", ListMarker::Ordered { start: 1, delimiter: ')' }, 2, Ok(ListEndReason::Reprocess))]
    #[case("1. b", ListMarker::Bullet('-'), 2, Ok(ListEndReason::Reprocess))]
    #[case("- b", ListMarker::Ordered { start: 1, delimiter: '.' }, 2, Ok(ListEndReason::Reprocess))]
    // 리스트 종료 (비리스트 내용)
    #[case("some text", ListMarker::Bullet('-'), 2, Ok(ListEndReason::Reprocess))]
    #[case("# heading", ListMarker::Bullet('-'), 2, Ok(ListEndReason::Reprocess))]
    #[case(
        "> blockquote",
        ListMarker::Bullet('-'),
        2,
        Ok(ListEndReason::Reprocess)
    )]
    #[case("```code", ListMarker::Bullet('-'), 2, Ok(ListEndReason::Reprocess))]
    fn test_try_end(
        #[case] line: &str,
        #[case] marker: ListMarker,
        #[case] content_indent: usize,
        #[case] expected: Result<ListEndReason, ListContinueReason>,
    ) {
        assert_eq!(try_end(line, &marker, content_indent), expected);
    }
}
