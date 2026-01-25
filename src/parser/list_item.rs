//! List Item 파서 (CommonMark 5.2 List Items)
//!
//! Bullet 마커 (-*+)와 Ordered 마커 (1. 1))를 감지합니다.
//! - 마커 인식 규칙 (Example 261, 265-269)
//! - 들여쓰기 규칙
//! - Continuation line 판별

use crate::node::ListType;
use super::helpers::count_leading_char;

// =============================================================================
// 타입 정의
// =============================================================================

/// 리스트 마커 타입
#[derive(Debug, Clone, PartialEq)]
pub enum ListMarker {
    /// Bullet 마커: '-', '+', '*'
    Bullet(char),
    /// Ordered 마커: 숫자 + '.' 또는 ')'
    Ordered {
        /// 시작 숫자
        start: usize,
        /// 구분자 ('.' 또는 ')')
        delimiter: char,
    },
}

impl ListMarker {
    /// ListType과 시작 번호로 변환
    pub fn to_list_type(&self) -> (ListType, usize) {
        match self {
            ListMarker::Bullet(_) => (ListType::Bullet, 1),
            ListMarker::Ordered { start, delimiter } => (
                ListType::Ordered {
                    delimiter: *delimiter,
                },
                *start,
            ),
        }
    }

    /// 같은 리스트 타입인지 확인 (같은 리스트에 속할 수 있는지)
    pub fn is_same_type(&self, other: &ListMarker) -> bool {
        match (self, other) {
            (ListMarker::Bullet(c1), ListMarker::Bullet(c2)) => c1 == c2,
            (
                ListMarker::Ordered { delimiter: d1, .. },
                ListMarker::Ordered { delimiter: d2, .. },
            ) => d1 == d2,
            _ => false,
        }
    }
}

/// List Item 시작 정보
/// try_start에서 반환되며, 같은 리스트 소속 여부 판단에 사용
#[derive(Debug, Clone, PartialEq)]
pub struct ListItemStart {
    /// 마커 타입
    pub marker: ListMarker,
    /// 마커 앞 들여쓰기 (0-3칸)
    pub indent: usize,
    /// 내용 시작 위치 (마커 + 공백 이후)
    pub content_indent: usize,
    /// 첫 줄 내용 (마커 이후)
    pub content: String,
}

impl ListItemStart {
    /// 라인에서 content를 추출하여 새 인스턴스 반환
    pub fn with_content_from(self, line: &str) -> Self {
        let content = if self.content_indent >= line.len() {
            String::new()
        } else {
            line[self.content_indent..].to_string()
        };
        Self { content, ..self }
    }

    #[cfg(test)]
    pub fn bullet(marker_char: char, indent: usize, content_indent: usize, content: &str) -> Self {
        Self {
            marker: ListMarker::Bullet(marker_char),
            indent,
            content_indent,
            content: content.to_string(),
        }
    }

    #[cfg(test)]
    pub fn ordered(
        start: usize,
        delimiter: char,
        indent: usize,
        content_indent: usize,
        content: &str,
    ) -> Self {
        Self {
            marker: ListMarker::Ordered { start, delimiter },
            indent,
            content_indent,
            content: content.to_string(),
        }
    }
}

/// List Item 시작 성공 사유
#[derive(Debug, Clone, PartialEq)]
pub enum ListItemStartReason {
    /// 정상적인 시작
    Started(ListItemStart),
}

/// List Item 시작 아님 사유
#[derive(Debug, Clone, PartialEq)]
pub enum ListItemNotStartReason {
    /// 4칸 이상 들여쓰기 (indented code block으로 해석됨)
    CodeBlockIndented,
    /// 유효한 리스트 마커 아님
    NotListMarker,
}

/// List 종료 사유
#[derive(Debug, Clone, PartialEq)]
pub enum ListEndReason {
    /// 줄 다시 처리 필요 (다른 블록/새 리스트)
    Reprocess,
}

/// 리스트 아이템 내용 줄
#[derive(Debug, Clone, PartialEq)]
pub struct ItemLine {
    /// 내용
    pub content: String,
    /// true면 텍스트 전용 (리스트 마커처럼 보여도 재파싱 시 리스트 아님)
    /// Example 303: 4칸 들여쓰기된 마커는 텍스트 전용
    pub text_only: bool,
}

impl ItemLine {
    pub fn new(content: String, text_only: bool) -> Self {
        Self { content, text_only }
    }

    pub fn text(content: String) -> Self {
        Self {
            content,
            text_only: false,
        }
    }

    pub fn text_only(content: String) -> Self {
        Self {
            content,
            text_only: true,
        }
    }

    pub fn blank() -> Self {
        Self {
            content: String::new(),
            text_only: false,
        }
    }
}

/// List 계속 사유
#[derive(Debug, Clone, PartialEq)]
pub enum ListContinueReason {
    /// 빈 줄 (pending_blank 설정)
    Blank,
    /// 새 아이템
    NewItem(ListItemStart),
    /// Continuation line (같은 아이템에 내용 추가)
    ContinuationLine(ItemLine),
}

// =============================================================================
// 함수
// =============================================================================

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
///
/// # Arguments
/// * `first_content_indent` - 첫 아이템의 content_indent (continuation 판단용)
/// * `current_content_indent` - 현재 아이템의 content_indent (새 아이템 판단용)
///
/// Example 301: 새 아이템 판단은 current_content_indent 기준 (0-3칸은 같은 레벨)
/// Example 303: continuation 판단은 first_content_indent 기준 (4칸 이상은 내용)
pub(crate) fn try_end(
    line: &str,
    marker: &ListMarker,
    first_content_indent: usize,
    current_content_indent: usize,
) -> Result<ListEndReason, ListContinueReason> {
    // 빈 줄 처리 (항상 계속, 개수는 호출자가 추적)
    if line.trim().is_empty() {
        return Err(ListContinueReason::Blank);
    }

    let indent = count_leading_char(line, ' ');

    // 1. 새 아이템 체크 (Example 301 지원)
    // current_content_indent 미만 들여쓰기에서만 새 아이템 가능
    if indent < current_content_indent {
        // 같은 마커 타입의 List Item이면 새 아이템으로 계속
        match try_start(line) {
            Ok(ListItemStartReason::Started(new_start)) => {
                if marker.is_same_type(&new_start.marker) {
                    return Err(ListContinueReason::NewItem(new_start));
                }
                // 다른 마커 타입이면 리스트 종료
                return Ok(ListEndReason::Reprocess);
            }
            Err(ListItemNotStartReason::CodeBlockIndented) => {
                // 4칸 이상 들여쓰기 → 리스트 마커로 인식 안 됨
                // 하지만 first_content_indent 이상이면 텍스트 전용 continuation
                if indent >= first_content_indent {
                    let strip_amount = indent.min(current_content_indent);
                    let content = line[strip_amount..].to_string();
                    // text_only: 재파싱 시 리스트로 인식 안 됨
                    return Err(ListContinueReason::ContinuationLine(ItemLine::text_only(content)));
                }
            }
            Err(ListItemNotStartReason::NotListMarker) => {
                // 리스트 마커가 아니면 continuation으로 넘어감
            }
        }
    }

    // 2. Continuation line (Example 303 지원)
    // first_content_indent 이상 들여쓰기는 현재 아이템의 내용
    if indent >= first_content_indent {
        // 내용 추출: min(indent, current_content_indent)만큼 제거
        // 예: current_content_indent=5이고 indent=4이면 4칸 제거
        // 예: current_content_indent=5이고 indent=6이면 5칸 제거
        let strip_amount = indent.min(current_content_indent);
        let content = line[strip_amount..].to_string();
        // 일반 continuation (중첩 리스트 가능)
        return Err(ListContinueReason::ContinuationLine(ItemLine::text(content)));
    }

    // 3. first_content_indent 미만 들여쓰기 + 새 아이템 아님 → 종료
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

    // 숫자가 없거나 9자리 초과면 실패 (Example 265-266)
    if num_str.is_empty() || num_str.len() > 9 {
        return None;
    }

    // 선행 0 허용: "003"은 3으로 파싱 (Example 267-268)
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
    // =========================================================================
    // Example 265-269: Ordered 마커 숫자 제약
    // =========================================================================
    // Example 265: 9자리까지 허용
    #[case(
        "123456789. ok",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(123456789, '.', 0, 11, "ok")))
    )]
    // Example 266: 10자리 이상은 마커 아님 (테스트는 에러 케이스 섹션에)
    // Example 267: 0으로 시작 가능
    #[case(
        "0. ok",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(0, '.', 0, 3, "ok")))
    )]
    // Example 268: 선행 0 허용 (값은 3)
    #[case(
        "003. ok",
        Ok(ListItemStartReason::Started(ListItemStart::ordered(3, '.', 0, 5, "ok")))
    )]
    // Example 269: 음수는 마커 아님 (테스트는 에러 케이스 섹션에)
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
    // Example 266: 10자리 이상은 마커 아님
    #[case("1234567890. not ok", Err(ListItemNotStartReason::NotListMarker))]
    // Example 269: 음수는 마커 아님 ('-'가 bullet 마커로 인식되지 않고 숫자도 아님)
    #[case("-1. not ok", Err(ListItemNotStartReason::NotListMarker))]
    #[case("a. item", Err(ListItemNotStartReason::NotListMarker))]
    #[case("1: item", Err(ListItemNotStartReason::NotListMarker))]
    fn test_try_start(
        #[case] input: &str,
        #[case] expected: Result<ListItemStartReason, ListItemNotStartReason>,
    ) {
        assert_eq!(try_start(input), expected);
    }

    // 5.2 List Items - 종료/계속 판별 (try_end)
    // 인자: (line, marker, first_content_indent, current_content_indent, expected)
    #[rstest]
    // 빈 줄 → Blank
    #[case("", ListMarker::Bullet('-'), 2, 2, Err(ListContinueReason::Blank))]
    #[case("  ", ListMarker::Bullet('-'), 2, 2, Err(ListContinueReason::Blank))]
    #[case("\t", ListMarker::Bullet('-'), 2, 2, Err(ListContinueReason::Blank))]
    // 같은 마커 → 새 아이템
    #[case(
        "- b",
        ListMarker::Bullet('-'),
        2, 2,
        Err(ListContinueReason::NewItem(ListItemStart::bullet('-', 0, 2, "b")))
    )]
    #[case(
        "+ b",
        ListMarker::Bullet('+'),
        2, 2,
        Err(ListContinueReason::NewItem(ListItemStart::bullet('+', 0, 2, "b")))
    )]
    #[case(
        "* b",
        ListMarker::Bullet('*'),
        2, 2,
        Err(ListContinueReason::NewItem(ListItemStart::bullet('*', 0, 2, "b")))
    )]
    #[case("2. b", ListMarker::Ordered { start: 1, delimiter: '.' }, 3, 3, Err(ListContinueReason::NewItem(ListItemStart::ordered(2, '.', 0, 3, "b"))))]
    #[case("2) b", ListMarker::Ordered { start: 1, delimiter: ')' }, 3, 3, Err(ListContinueReason::NewItem(ListItemStart::ordered(2, ')', 0, 3, "b"))))]
    // Continuation line (text_only=false: 일반 continuation)
    #[case("  continued", ListMarker::Bullet('-'), 2, 2, Err(ListContinueReason::ContinuationLine(ItemLine::text("continued".to_string()))))]
    #[case("   continued", ListMarker::Bullet('-'), 2, 2, Err(ListContinueReason::ContinuationLine(ItemLine::text(" continued".to_string()))))]
    #[case("    continued", ListMarker::Bullet('-'), 2, 2, Err(ListContinueReason::ContinuationLine(ItemLine::text("  continued".to_string()))))]
    // 리스트 종료 (다른 마커)
    #[case("+ b", ListMarker::Bullet('-'), 2, 2, Ok(ListEndReason::Reprocess))]
    #[case("* b", ListMarker::Bullet('-'), 2, 2, Ok(ListEndReason::Reprocess))]
    #[case("- b", ListMarker::Bullet('+'), 2, 2, Ok(ListEndReason::Reprocess))]
    #[case("1) b", ListMarker::Ordered { start: 1, delimiter: '.' }, 3, 3, Ok(ListEndReason::Reprocess))]
    #[case("1. b", ListMarker::Ordered { start: 1, delimiter: ')' }, 3, 3, Ok(ListEndReason::Reprocess))]
    #[case("1. b", ListMarker::Bullet('-'), 2, 2, Ok(ListEndReason::Reprocess))]
    #[case("- b", ListMarker::Ordered { start: 1, delimiter: '.' }, 3, 3, Ok(ListEndReason::Reprocess))]
    // 리스트 종료 (비리스트 내용)
    #[case("some text", ListMarker::Bullet('-'), 2, 2, Ok(ListEndReason::Reprocess))]
    #[case("# heading", ListMarker::Bullet('-'), 2, 2, Ok(ListEndReason::Reprocess))]
    #[case(
        "> blockquote",
        ListMarker::Bullet('-'),
        2, 2,
        Ok(ListEndReason::Reprocess)
    )]
    #[case("```code", ListMarker::Bullet('-'), 2, 2, Ok(ListEndReason::Reprocess))]
    // Example 303: 4칸 들여쓰기된 마커는 continuation (first=2, current=5)
    // "   - d"의 content_indent=5, "    - e"(4칸)는 continuation이어야 함
    // 내용: min(4, 5)=4칸 제거 → "- e", text_only=true (리스트 마커 아님)
    #[case("    - e", ListMarker::Bullet('-'), 2, 5, Err(ListContinueReason::ContinuationLine(ItemLine::text_only("- e".to_string()))))]
    fn test_try_end(
        #[case] line: &str,
        #[case] marker: ListMarker,
        #[case] first_content_indent: usize,
        #[case] current_content_indent: usize,
        #[case] expected: Result<ListEndReason, ListContinueReason>,
    ) {
        assert_eq!(try_end(line, &marker, first_content_indent, current_content_indent), expected);
    }

    // === ListMarker::to_list_type 테스트 ===
    #[rstest]
    #[case(ListMarker::Bullet('-'), ListType::Bullet, 1)]
    #[case(ListMarker::Bullet('+'), ListType::Bullet, 1)]
    #[case(ListMarker::Bullet('*'), ListType::Bullet, 1)]
    #[case(ListMarker::Ordered { start: 1, delimiter: '.' }, ListType::Ordered { delimiter: '.' }, 1)]
    #[case(ListMarker::Ordered { start: 5, delimiter: '.' }, ListType::Ordered { delimiter: '.' }, 5)]
    #[case(ListMarker::Ordered { start: 1, delimiter: ')' }, ListType::Ordered { delimiter: ')' }, 1)]
    fn test_to_list_type(
        #[case] marker: ListMarker,
        #[case] expected_type: ListType,
        #[case] expected_start: usize,
    ) {
        let (list_type, start) = marker.to_list_type();
        assert_eq!(list_type, expected_type);
        assert_eq!(start, expected_start);
    }

    // === ListMarker::is_same_type 테스트 ===
    #[rstest]
    // 같은 Bullet 마커
    #[case(ListMarker::Bullet('-'), ListMarker::Bullet('-'), true)]
    #[case(ListMarker::Bullet('+'), ListMarker::Bullet('+'), true)]
    #[case(ListMarker::Bullet('*'), ListMarker::Bullet('*'), true)]
    // 다른 Bullet 마커
    #[case(ListMarker::Bullet('-'), ListMarker::Bullet('+'), false)]
    #[case(ListMarker::Bullet('-'), ListMarker::Bullet('*'), false)]
    // 같은 Ordered 마커 (delimiter만 비교, start는 무관)
    #[case(ListMarker::Ordered { start: 1, delimiter: '.' }, ListMarker::Ordered { start: 1, delimiter: '.' }, true)]
    #[case(ListMarker::Ordered { start: 1, delimiter: '.' }, ListMarker::Ordered { start: 5, delimiter: '.' }, true)]
    #[case(ListMarker::Ordered { start: 1, delimiter: ')' }, ListMarker::Ordered { start: 1, delimiter: ')' }, true)]
    // 다른 Ordered 마커
    #[case(ListMarker::Ordered { start: 1, delimiter: '.' }, ListMarker::Ordered { start: 1, delimiter: ')' }, false)]
    // Bullet과 Ordered 혼합
    #[case(ListMarker::Bullet('-'), ListMarker::Ordered { start: 1, delimiter: '.' }, false)]
    fn test_is_same_type(#[case] a: ListMarker, #[case] b: ListMarker, #[case] expected: bool) {
        assert_eq!(a.is_same_type(&b), expected);
    }
}
