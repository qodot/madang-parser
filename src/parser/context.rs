//! 파싱 컨텍스트 타입 정의
//!
//! 시작 정보(Start)와 파싱 상태(ParsingContext)를 분리하여 관리합니다.

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
}
