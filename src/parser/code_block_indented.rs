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

    /// 통합 테스트: CommonMark 명세 예제 기반
    mod integration {
        use crate::node::Node;
        use crate::parse;
        use rstest::rstest;

        /// CommonMark Indented Code Block 예제 테스트
        #[rstest]
        // Example 107: 기본 코드 블록
        #[case("    a simple\n      indented code block", "a simple\n  indented code block")]
        // Example 110: HTML/마크다운은 그대로 코드로 처리
        #[case("    <a/>\n    *hi*\n\n    - one", "<a/>\n*hi*\n\n- one")]
        // Example 111: 빈 줄로 분리된 청크들은 하나의 블록
        #[case("    chunk1\n\n    chunk2\n  \n \n \n    chunk3", "chunk1\n\nchunk2\n\n\n\nchunk3")]
        // Example 112: 들여쓰기된 빈 줄 유지
        #[case("    chunk1\n      \n      chunk2", "chunk1\n  \n  chunk2")]
        // Example 116: 8칸 들여쓰기 (4칸 제거 후 4칸 유지)
        #[case("        foo\n    bar", "    foo\nbar")]
        // Example 117: 앞뒤 빈 줄은 제거됨
        #[case("\n    \n    foo\n    ", "foo")]
        // Example 118: 후행 공백은 유지됨
        #[case("    foo  ", "foo  ")]
        fn test_code_block_indented(#[case] input: &str, #[case] expected_content: &str) {
            let doc = parse(input);
            let Node::Document { children } = doc else {
                panic!("expected Document");
            };
            assert_eq!(children.len(), 1, "children: {:?}", children);
            let Node::CodeBlock { info, content } = &children[0] else {
                panic!("expected CodeBlock, got {:?}", children[0]);
            };
            assert_eq!(*info, None);
            assert_eq!(content, expected_content, "input: {:?}", input);
        }

        /// Example 113: Paragraph 인터럽트 불가
        /// 빈 줄 없이 4칸 들여쓰기는 Paragraph의 일부
        #[rstest]
        #[case("Foo\n    bar", "Foo\nbar")]
        fn test_cannot_interrupt_paragraph(#[case] input: &str, #[case] expected_text: &str) {
            let doc = parse(input);
            let Node::Document { children } = doc else {
                panic!("expected Document");
            };
            assert_eq!(children.len(), 1, "children: {:?}", children);
            let Node::Paragraph { children: para_children } = &children[0] else {
                panic!("expected Paragraph, got {:?}", children[0]);
            };
            assert_eq!(para_children.len(), 1);
            assert_eq!(para_children[0].as_text(), expected_text);
        }

        /// Example 114: 코드 블록 후 4칸 미만 줄은 새 Paragraph
        #[rstest]
        #[case("    foo\nbar", "foo", "bar")]
        fn test_code_then_paragraph(
            #[case] input: &str,
            #[case] code_content: &str,
            #[case] para_text: &str,
        ) {
            let doc = parse(input);
            let Node::Document { children } = doc else {
                panic!("expected Document");
            };
            assert_eq!(children.len(), 2, "children: {:?}", children);
            // 첫 번째: CodeBlock
            let Node::CodeBlock { info, content } = &children[0] else {
                panic!("expected CodeBlock, got {:?}", children[0]);
            };
            assert_eq!(*info, None);
            assert_eq!(content, code_content);
            // 두 번째: Paragraph
            let Node::Paragraph { children: para_children } = &children[1] else {
                panic!("expected Paragraph, got {:?}", children[1]);
            };
            assert_eq!(para_children[0].as_text(), para_text);
        }
    }
}
