//! 파싱 컨텍스트 타입 정의
//!
//! 시작 정보(Start)와 파싱 상태(ParsingContext)를 분리하여 관리합니다.

use crate::node::ListType;

// =============================================================================
// Fenced Code Block
// =============================================================================

/// Fenced Code Block 시작 정보
/// try_start에서 반환되며, 종료 조건 판단에 사용
#[derive(Debug, Clone)]
pub struct CodeBlockFencedStart {
    /// 펜스 문자 ('`' 또는 '~')
    pub fence_char: char,
    /// 펜스 길이 (최소 3)
    pub fence_len: usize,
    /// info string (언어 등)
    pub info: Option<String>,
    /// 여는 펜스의 들여쓰기
    pub indent: usize,
}

/// Fenced Code Block 시작 성공 사유
#[derive(Debug, Clone)]
pub enum CodeBlockFencedStartReason {
    /// 정상적인 시작
    Started(CodeBlockFencedStart),
}

/// Fenced Code Block 시작 아님 사유
#[derive(Debug, Clone, PartialEq)]
pub enum CodeBlockFencedNotStartReason {
    /// 4칸 이상 들여쓰기 (indented code block으로 해석됨)
    CodeBlockIndented,
    /// 펜스 문자 없음 (```, ~~~가 아님)
    NoFence,
}

/// Fenced Code Block 종료 사유
#[derive(Debug, Clone, PartialEq)]
pub enum CodeBlockFencedEndReason {
    /// 닫는 펜스 발견
    ClosingFence,
}

/// Fenced Code Block 계속 사유
#[derive(Debug, Clone, PartialEq)]
pub enum CodeBlockFencedContinueReason {
    /// 4칸 이상 들여쓰기 (코드 내용)
    TooMuchIndent,
    /// 펜스 길이 부족
    FenceTooShort,
    /// 펜스 문자 불일치
    FenceCharMismatch,
    /// 펜스 뒤 텍스트 있음
    TextAfterFence,
}

// =============================================================================
// List
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
            (ListMarker::Ordered { delimiter: d1, .. }, ListMarker::Ordered { delimiter: d2, .. }) => {
                d1 == d2
            }
            _ => false,
        }
    }
}

/// List Item 시작 정보
/// try_start에서 반환되며, 같은 리스트 소속 여부 판단에 사용
#[derive(Debug, Clone)]
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
}

/// List Item 시작 성공 사유
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub enum ListEndReason {
    /// 줄 다시 처리 필요 (다른 블록/새 리스트)
    Reprocess,
}

/// List 계속 사유
#[derive(Debug, Clone)]
pub enum ListContinueReason {
    /// 빈 줄 (pending_blank 설정)
    Blank,
    /// 새 아이템
    NewItem(ListItemStart),
    /// Continuation line (같은 아이템에 내용 추가)
    ContinuationLine(String),
}

// =============================================================================
// Indented Code Block
// =============================================================================

/// Indented Code Block 시작 정보
#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlockIndentedStart {
    /// 첫 줄 내용 (4칸 들여쓰기 제거 후)
    pub content: String,
}

/// Indented Code Block 시작 성공 사유
#[derive(Debug, Clone, PartialEq)]
pub enum CodeBlockIndentedStartReason {
    /// 정상적인 시작
    Started(CodeBlockIndentedStart),
}

/// Indented Code Block 시작 아님 사유
#[derive(Debug, Clone, PartialEq)]
pub enum CodeBlockIndentedNotStartReason {
    /// 빈 줄 (공백만 있는 줄 포함)
    Empty,
    /// 들여쓰기 부족 (4칸 미만)
    InsufficientIndent,
}

// =============================================================================
// Setext Heading
// =============================================================================

/// Setext 밑줄 레벨
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SetextLevel {
    /// `=` 밑줄 → 레벨 1
    Level1,
    /// `-` 밑줄 → 레벨 2
    Level2,
}

impl SetextLevel {
    /// 숫자 레벨로 변환 (Heading의 level 필드용)
    pub fn to_level(self) -> u8 {
        match self {
            SetextLevel::Level1 => 1,
            SetextLevel::Level2 => 2,
        }
    }
}

/// Setext Heading 밑줄 시작 정보
#[derive(Debug, Clone, PartialEq)]
pub struct HeadingSetextStart {
    /// 밑줄 레벨 (1 또는 2)
    pub level: SetextLevel,
}

/// Setext 밑줄 감지 성공 사유
#[derive(Debug, Clone, PartialEq)]
pub enum HeadingSetextStartReason {
    /// 유효한 밑줄 발견
    Started(HeadingSetextStart),
}

/// Setext 밑줄 아님 사유
#[derive(Debug, Clone, PartialEq)]
pub enum HeadingSetextNotStartReason {
    /// 4칸 이상 들여쓰기 (코드 블록으로 해석됨)
    CodeBlockIndented,
    /// 빈 줄
    Empty,
    /// 밑줄 문자(=, -)가 아님
    NotUnderlineChar,
    /// 문자가 섞임 (예: "=-=")
    MixedChars,
}

// =============================================================================
// Parsing Context
// =============================================================================

/// 파싱 중인 컨텍스트 (상태 기계의 상태)
pub enum ParsingContext {
    /// 새 블록 시작 대기
    None,

    /// Fenced Code Block 파싱 중
    CodeBlockFenced {
        /// 시작 정보 (불변)
        start: CodeBlockFencedStart,
        /// 축적된 코드 줄 (가변)
        content: Vec<String>,
    },

    /// Paragraph 파싱 중 (여러 줄이 하나의 문단)
    Paragraph { lines: Vec<String> },

    /// Blockquote 파싱 중 (여러 줄 수집)
    Blockquote { lines: Vec<String> },

    /// List 파싱 중
    List {
        /// 첫 아이템의 시작 정보 (리스트 타입 결정용)
        first_item_start: ListItemStart,
        /// 완성된 아이템들의 내용
        items: Vec<Vec<String>>,
        /// 현재 아이템의 줄들
        current_item_lines: Vec<String>,
        /// tight 리스트 여부 (아이템 간 빈 줄 없음)
        tight: bool,
        /// 대기 중인 빈 줄 개수 (continuation 시 내용에 추가)
        pending_blank_count: usize,
    },

    /// Indented Code Block 파싱 중
    CodeBlockIndented {
        /// 축적된 코드 줄 (4칸 들여쓰기 제거 후)
        lines: Vec<String>,
        /// 대기 중인 빈 줄 개수 (다음 코드 줄이 오면 내용에 추가)
        pending_blank_count: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

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

    // === SetextLevel::to_level 테스트 ===
    #[rstest]
    #[case(SetextLevel::Level1, 1)]
    #[case(SetextLevel::Level2, 2)]
    fn test_setext_level_to_level(#[case] level: SetextLevel, #[case] expected: u8) {
        assert_eq!(level.to_level(), expected);
    }
}
