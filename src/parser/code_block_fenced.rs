//! https://spec.commonmark.org/0.31.2/#fenced-code-blocks

use crate::node::{BlockNode, CodeBlockNode};
use super::helpers::{count_leading_char, remove_indent};

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum CodeBlockFencedOk {
    /// 시작 줄 (여는 펜스)
    Start(CodeBlockFencedStart),
    /// 내용 줄 (들여쓰기 제거됨)
    Content(String),
    /// 종료 줄 (닫는 펜스)
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CodeBlockFencedErr {
    /// 4칸 이상 들여쓰기 (indented code block으로 해석됨)
    CodeBlockIndented,
    /// 펜스 문자 없음 (```, ~~~가 아님)
    NoFence,
}

pub fn parse(line: &str, start: Option<&CodeBlockFencedStart>) -> Result<CodeBlockFencedOk, CodeBlockFencedErr> {
    match start {
        None => parse_start(line),
        Some(s) => Ok(parse_continue(line, s)),
    }
}

fn parse_start(line: &str) -> Result<CodeBlockFencedOk, CodeBlockFencedErr> {
    let indent = count_leading_char(line, ' ');

    // 4칸 이상 들여쓰기는 indented code block
    if indent > 3 {
        return Err(CodeBlockFencedErr::CodeBlockIndented);
    }

    let after_indent = &line[indent..];

    // 펜스 문자와 길이 확인
    let (fence_char, fence_len) = if after_indent.starts_with("```") {
        ('`', count_leading_char(after_indent, '`'))
    } else if after_indent.starts_with("~~~") {
        ('~', count_leading_char(after_indent, '~'))
    } else {
        return Err(CodeBlockFencedErr::NoFence);
    };

    // info string 추출
    let info = {
        let after_fence = &after_indent[fence_len..];
        let trimmed = after_fence.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    };

    Ok(CodeBlockFencedOk::Start(CodeBlockFencedStart {
        fence_char,
        fence_len,
        info,
        indent,
    }))
}

fn parse_continue(line: &str, start: &CodeBlockFencedStart) -> CodeBlockFencedOk {
    let indent = count_leading_char(line, ' ');

    // 4칸 이상 들여쓰기는 내용
    if indent > 3 {
        return CodeBlockFencedOk::Content(remove_indent(line, start.indent).to_string());
    }

    let after_indent = &line[indent..];
    let closing_len = count_leading_char(after_indent, start.fence_char);

    // 닫는 펜스 조건: 같은 문자, 충분한 길이, 뒤에 텍스트 없음
    if closing_len >= start.fence_len && after_indent[closing_len..].trim().is_empty() {
        return CodeBlockFencedOk::End;
    }

    CodeBlockFencedOk::Content(remove_indent(line, start.indent).to_string())
}

pub fn finalize(start: CodeBlockFencedStart, content: Vec<String>) -> BlockNode {
    let content_str = content.join("\n");
    BlockNode::CodeBlock(CodeBlockNode::new(start.info, content_str))
}

pub fn parse_text(text: &str) -> Option<BlockNode> {
    let lines: Vec<&str> = text.lines().collect();

    if lines.is_empty() {
        return None;
    }

    // 여는 펜스 확인
    let first_line = lines[0];
    let start = match parse(first_line, None) {
        Ok(CodeBlockFencedOk::Start(s)) => s,
        _ => return None,
    };

    // 닫는 펜스 찾기
    let has_closing_fence = if lines.len() >= 2 {
        let last_line = lines[lines.len() - 1];
        matches!(parse(last_line, Some(&start)), Ok(CodeBlockFencedOk::End))
    } else {
        false
    };

    // 내용 추출 (들여쓰기 제거)
    let content_lines: Vec<&str> = if has_closing_fence {
        lines[1..lines.len() - 1].to_vec()
    } else {
        lines[1..].to_vec()
    };

    let content: Vec<String> = content_lines
        .iter()
        .map(|line| remove_indent(line, start.indent).to_string())
        .collect();

    Some(finalize(start, content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::BlockNode;
    use rstest::rstest;

    // =============================================================================
    // parse(line, None) 테스트 - 시작 줄 판단
    // =============================================================================

    #[rstest]
    // Example 119: 백틱 펜스
    #[case("```", Some(('`', 3, None, 0)))]
    #[case("````", Some(('`', 4, None, 0)))]
    #[case("`````", Some(('`', 5, None, 0)))]
    // Example 120: 틸드 펜스
    #[case("~~~", Some(('~', 3, None, 0)))]
    #[case("~~~~", Some(('~', 4, None, 0)))]
    #[case("~~~~~", Some(('~', 5, None, 0)))]
    // Example 142-143: info string
    #[case("```rust", Some(('`', 3, Some("rust"), 0)))]
    #[case("~~~ python", Some(('~', 3, Some("python"), 0)))]
    #[case("```  rust  ", Some(('`', 3, Some("rust"), 0)))]
    #[case("```rust python", Some(('`', 3, Some("rust python"), 0)))]
    // Example 131-133: 들여쓰기 0-3칸
    #[case(" ```", Some(('`', 3, None, 1)))]
    #[case("  ```", Some(('`', 3, None, 2)))]
    #[case("   ```", Some(('`', 3, None, 3)))]
    #[case("   ```rust", Some(('`', 3, Some("rust"), 3)))]
    // 펜스가 아닌 경우
    #[case("``", None)]
    #[case("~~", None)]
    #[case("    ```", None)]  // 4칸 들여쓰기
    #[case("code", None)]
    #[case("", None)]
    #[case("  ", None)]
    fn test_parse_start(
        #[case] input: &str,
        #[case] expected: Option<(char, usize, Option<&str>, usize)>,
    ) {
        let result = parse(input, None);
        match expected {
            Some((expected_char, expected_len, expected_info, expected_indent)) => {
                if let Ok(CodeBlockFencedOk::Start(start)) = result {
                    assert_eq!(start.fence_char, expected_char, "fence_char");
                    assert_eq!(start.fence_len, expected_len, "fence_len");
                    assert_eq!(start.info.as_deref(), expected_info, "info");
                    assert_eq!(start.indent, expected_indent, "indent");
                } else {
                    panic!("시작이어야 함: {:?}, got {:?}", input, result);
                }
            }
            None => {
                assert!(
                    result.is_err(),
                    "시작이 아니어야 함: {:?}, got {:?}",
                    input,
                    result
                );
            }
        }
    }

    // =============================================================================
    // parse(line, Some(&start)) 테스트 - 종료/내용 판단
    // =============================================================================

    #[rstest]
    // Example 124-125: 유효한 닫는 펜스
    #[case("```", '`', 3, true)]
    #[case("````", '`', 3, true)]
    #[case("`````", '`', 3, true)]
    #[case("~~~", '~', 3, true)]
    #[case("~~~~", '~', 3, true)]
    // 들여쓰기 0-3칸
    #[case(" ```", '`', 3, true)]
    #[case("  ```", '`', 3, true)]
    #[case("   ```", '`', 3, true)]
    // 펜스 뒤 공백만 허용
    #[case("```  ", '`', 3, true)]
    #[case("~~~   ", '~', 3, true)]
    // Example 139: 유효하지 않은 닫는 펜스 → Content
    #[case("``", '`', 3, false)]
    #[case("```", '`', 4, false)]
    // Example 122-123: 문자 불일치
    #[case("~~~", '`', 3, false)]
    #[case("```", '~', 3, false)]
    // 4칸 들여쓰기
    #[case("    ```", '`', 3, false)]
    // Example 140: 펜스 뒤 텍스트
    #[case("```code", '`', 3, false)]
    #[case("``` x", '`', 3, false)]
    fn test_parse_continue(
        #[case] input: &str,
        #[case] fence_char: char,
        #[case] fence_len: usize,
        #[case] is_end: bool,
    ) {
        let start = CodeBlockFencedStart {
            fence_char,
            fence_len,
            info: None,
            indent: 0,
        };
        let result = parse(input, Some(&start));
        if is_end {
            assert!(
                matches!(result, Ok(CodeBlockFencedOk::End)),
                "종료여야 함: {:?}, got {:?}",
                input,
                result
            );
        } else {
            assert!(
                matches!(result, Ok(CodeBlockFencedOk::Content(_))),
                "내용이어야 함: {:?}, got {:?}",
                input,
                result
            );
        }
    }

    // =============================================================================
    // 통합 테스트 (CommonMark Example 기반)
    // =============================================================================

    #[rstest]
    // Example 119-120: 기본 백틱/틸드 펜스
    #[case("```\ncode\n```", vec![BlockNode::code_block(None, "code")])]
    #[case("~~~\ncode\n~~~", vec![BlockNode::code_block(None, "code")])]
    #[case("```\nline1\nline2\n```", vec![BlockNode::code_block(None, "line1\nline2")])]
    // Example 122-123: 다른 문자 펜스는 닫히지 않음
    #[case("~~~\ncode\n```", vec![BlockNode::code_block(None, "code\n```")])]
    #[case("```\ncode\n~~~", vec![BlockNode::code_block(None, "code\n~~~")])]
    // Example 124-125: 닫는 펜스가 더 길어도 OK
    #[case("`````\ncode\n`````", vec![BlockNode::code_block(None, "code")])]
    #[case("```\ncode\n`````", vec![BlockNode::code_block(None, "code")])]
    #[case("~~~~~\ncode\n~~~~~", vec![BlockNode::code_block(None, "code")])]
    // Example 126, 130: 빈 내용
    #[case("```\n\n```", vec![BlockNode::code_block(None, "")])]
    // Example 127: 닫는 펜스 없음 → EOF까지
    #[case("```\ncode", vec![BlockNode::code_block(None, "code")])]
    #[case("```rust\ncode", vec![BlockNode::code_block(Some("rust"), "code")])]
    #[case("```\nline1\nline2", vec![BlockNode::code_block(None, "line1\nline2")])]
    #[case("~~~\ncode", vec![BlockNode::code_block(None, "code")])]
    // Example 131-133: 여는 펜스 들여쓰기 처리
    #[case("  ```\n  code\n  ```", vec![BlockNode::code_block(None, "code")])]
    #[case("   ```\n   code\n   ```", vec![BlockNode::code_block(None, "code")])]
    #[case("  ```\n    code\n  ```", vec![BlockNode::code_block(None, "  code")])]
    #[case("  ```\ncode\n  ```", vec![BlockNode::code_block(None, "code")])]
    // Example 139: 닫는 펜스가 짧으면 무효
    #[case("`````\ncode\n```", vec![BlockNode::code_block(None, "code\n```")])]
    #[case("~~~~~\ncode\n~~~", vec![BlockNode::code_block(None, "code\n~~~")])]
    // Example 142-143: info string
    #[case("```rust\ncode\n```", vec![BlockNode::code_block(Some("rust"), "code")])]
    #[case("``` rust \ncode\n```", vec![BlockNode::code_block(Some("rust"), "code")])]
    #[case("```rust python\ncode\n```", vec![BlockNode::code_block(Some("rust python"), "code")])]
    #[case("~~~rust\ncode\n~~~", vec![BlockNode::code_block(Some("rust"), "code")])]
    // Example 144: 특수 문자 info string
    #[case("```;\n````", vec![BlockNode::code_block(Some(";"), "")])]
    // 추가: 빈 줄 포함
    #[case("```\nline1\n\nline2\n```", vec![BlockNode::code_block(None, "line1\n\nline2")])]
    #[case("```rust\nfn main() {\n\n    println!(\"hi\");\n}\n```", vec![BlockNode::code_block(Some("rust"), "fn main() {\n\n    println!(\"hi\");\n}")])]
    fn test_code_block_fenced(#[case] input: &str, #[case] expected: Vec<BlockNode>) {
        let doc = crate::parse(input);
        assert_eq!(doc.children, expected);
    }
}
