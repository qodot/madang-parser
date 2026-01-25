//! Indented Code Block 파서
//!
//! 4칸 들여쓰기로 작성된 코드 블록을 파싱합니다.

use super::context::{
    CodeBlockIndentedNotStartReason, CodeBlockIndentedStart, CodeBlockIndentedStartReason,
};
use super::helpers::count_leading_char;

/// Indented Code Block 시작 줄인지 확인
/// 성공 시 Ok(Started), 실패 시 Err(사유) 반환
pub(crate) fn try_start(
    line: &str,
) -> Result<CodeBlockIndentedStartReason, CodeBlockIndentedNotStartReason> {
    // 1. 들여쓰기 확인 (4칸 이상이면 코드 줄)
    let indent = count_leading_char(line, ' ');
    if indent >= 4 {
        // 4칸 제거 후 내용 반환 (공백만 있는 줄도 코드의 일부)
        let content = line[4..].to_string();
        return Ok(CodeBlockIndentedStartReason::Started(
            CodeBlockIndentedStart { content },
        ));
    }

    // 2. 4칸 미만 들여쓰기: 빈 줄이면 Empty, 아니면 InsufficientIndent
    if line.trim().is_empty() {
        return Err(CodeBlockIndentedNotStartReason::Empty);
    }

    Err(CodeBlockIndentedNotStartReason::InsufficientIndent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{BlockNode, InlineNode};
    use rstest::rstest;

    /// try_start 테스트: 성공/실패 케이스 통합
    /// expected: Ok(content) 또는 Err(reason)
    #[rstest]
    // 성공 케이스: 4칸 이상 들여쓰기
    #[case("    code", Ok("code"))]
    #[case("     code", Ok(" code"))]
    #[case("        code", Ok("    code"))]
    // 실패 케이스: 빈 줄
    #[case("", Err(CodeBlockIndentedNotStartReason::Empty))]
    #[case("   ", Err(CodeBlockIndentedNotStartReason::Empty))]
    // 실패 케이스: 들여쓰기 부족
    #[case("code", Err(CodeBlockIndentedNotStartReason::InsufficientIndent))]
    #[case(" code", Err(CodeBlockIndentedNotStartReason::InsufficientIndent))]
    #[case("  code", Err(CodeBlockIndentedNotStartReason::InsufficientIndent))]
    #[case("   code", Err(CodeBlockIndentedNotStartReason::InsufficientIndent))]
    fn test_try_start(
        #[case] input: &str,
        #[case] expected: Result<&str, CodeBlockIndentedNotStartReason>,
    ) {
        let result = try_start(input);
        match expected {
            Ok(content) => {
                let reason = result.expect("시작이어야 함");
                let CodeBlockIndentedStartReason::Started(start) = reason;
                assert_eq!(start.content, content, "입력: {:?}", input);
            }
            Err(expected_reason) => {
                let reason = result.expect_err("시작이 아니어야 함");
                assert_eq!(reason, expected_reason, "입력: {:?}", input);
            }
        }
    }

    /// Indented Code Block 통합 테스트 (CommonMark 명세 기반)
    #[rstest]
    // Example 107: 기본 코드 블록
    #[case("    a simple\n      indented code block", vec![BlockNode::code_block(None, "a simple\n  indented code block")])]
    // Example 110: HTML/마크다운은 그대로 코드로 처리
    #[case("    <a/>\n    *hi*\n\n    - one", vec![BlockNode::code_block(None, "<a/>\n*hi*\n\n- one")])]
    // Example 111: 빈 줄로 분리된 청크들은 하나의 블록
    #[case("    chunk1\n\n    chunk2\n  \n \n \n    chunk3", vec![BlockNode::code_block(None, "chunk1\n\nchunk2\n\n\n\nchunk3")])]
    // Example 112: 들여쓰기된 빈 줄 유지
    #[case("    chunk1\n      \n      chunk2", vec![BlockNode::code_block(None, "chunk1\n  \n  chunk2")])]
    // Example 113: Paragraph 인터럽트 불가 - 빈 줄 없이 4칸 들여쓰기는 Paragraph 일부
    #[case("Foo\n    bar", vec![BlockNode::paragraph(vec![InlineNode::text("Foo\nbar")])])]
    // Example 114: 코드 블록 후 4칸 미만 줄은 새 Paragraph
    #[case("    foo\nbar", vec![BlockNode::code_block(None, "foo"), BlockNode::paragraph(vec![InlineNode::text("bar")])])]
    // Example 116: 8칸 들여쓰기 (4칸 제거 후 4칸 유지)
    #[case("        foo\n    bar", vec![BlockNode::code_block(None, "    foo\nbar")])]
    // Example 117: 앞뒤 빈 줄은 제거됨
    #[case("\n    \n    foo\n    ", vec![BlockNode::code_block(None, "foo")])]
    // Example 118: 후행 공백은 유지됨
    #[case("    foo  ", vec![BlockNode::code_block(None, "foo  ")])]
    fn test_code_block_indented(#[case] input: &str, #[case] expected: Vec<BlockNode>) {
        let doc = crate::parse(input);
        assert_eq!(doc.children, expected);
    }
}
