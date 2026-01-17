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
pub struct FencedCodeBlockStart {
    /// 펜스 문자 ('`' 또는 '~')
    pub fence_char: char,
    /// 펜스 길이 (최소 3)
    pub fence_len: usize,
    /// info string (언어 등)
    pub info: Option<String>,
    /// 여는 펜스의 들여쓰기
    pub indent: usize,
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

// =============================================================================
// Parsing Context
// =============================================================================

/// 파싱 중인 컨텍스트 (상태 기계의 상태)
pub enum ParsingContext {
    /// 새 블록 시작 대기
    None,

    /// Fenced Code Block 파싱 중
    FencedCodeBlock {
        /// 시작 정보 (불변)
        start: FencedCodeBlockStart,
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
    },
}
