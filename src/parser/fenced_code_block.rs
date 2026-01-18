//! Fenced Code Block 파서
//!
//! 백틱(\`\`\`) 또는 틸드(~~~)로 감싸진 코드 블록을 파싱합니다.

use crate::node::Node;
use super::context::{
    FencedCodeBlockContinueReason, FencedCodeBlockEndReason, FencedCodeBlockNotStartReason,
    FencedCodeBlockStart, FencedCodeBlockStartReason,
};
use super::helpers::{count_leading_char, remove_indent};

/// Fenced Code Block 시작 줄인지 확인
/// 성공 시 Ok(Started), 실패 시 Err(사유) 반환
pub(crate) fn try_start(line: &str) -> Result<FencedCodeBlockStartReason, FencedCodeBlockNotStartReason> {
    let indent = count_leading_char(line, ' ');
    if indent > 3 {
        return Err(FencedCodeBlockNotStartReason::IndentedCodeBlock);
    }

    let after_indent = &line[indent..];

    let (fence_char, fence_len) = if after_indent.starts_with("```") {
        ('`', count_leading_char(after_indent, '`'))
    } else if after_indent.starts_with("~~~") {
        ('~', count_leading_char(after_indent, '~'))
    } else {
        return Err(FencedCodeBlockNotStartReason::NoFence);
    };

    let info = {
        let after_fence = &after_indent[fence_len..];
        let trimmed = after_fence.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    };

    Ok(FencedCodeBlockStartReason::Started(FencedCodeBlockStart {
        fence_char,
        fence_len,
        info,
        indent,
    }))
}

/// 닫는 펜스인지 확인
/// 성공 시 Ok(ClosingFence), 실패 시 Err(사유) 반환
pub(crate) fn try_end(
    line: &str,
    fence_char: char,
    min_fence_len: usize,
) -> Result<FencedCodeBlockEndReason, FencedCodeBlockContinueReason> {
    let indent = count_leading_char(line, ' ');
    if indent > 3 {
        return Err(FencedCodeBlockContinueReason::TooMuchIndent);
    }

    let after_indent = &line[indent..];
    let closing_len = count_leading_char(after_indent, fence_char);

    // 펜스 문자 없음 (다른 문자로 시작)
    if closing_len == 0 {
        return Err(FencedCodeBlockContinueReason::FenceCharMismatch);
    }

    // 펜스 길이 부족
    if closing_len < min_fence_len {
        return Err(FencedCodeBlockContinueReason::FenceTooShort);
    }

    // 펜스 뒤 텍스트 있음
    if !after_indent[closing_len..].trim().is_empty() {
        return Err(FencedCodeBlockContinueReason::TextAfterFence);
    }

    Ok(FencedCodeBlockEndReason::ClosingFence)
}

/// Fenced Code Block 파싱 시도 (블록 단위)
/// blockquote 내부 등에서 사용
/// 성공하면 Some(CodeBlock), 실패하면 None 반환
pub fn parse(text: &str, _indent: usize) -> Option<Node> {
    let lines: Vec<&str> = text.lines().collect();

    if lines.is_empty() {
        return None;
    }

    // 여는 펜스의 들여쓰기 계산 (0-3칸만 허용)
    let first_line = lines[0];
    let start = match try_start(first_line) {
        Ok(FencedCodeBlockStartReason::Started(s)) => s,
        Err(_) => return None,
    };

    // 닫는 펜스 찾기
    let has_closing_fence = if lines.len() >= 2 {
        let last_line = lines[lines.len() - 1];
        try_end(last_line, start.fence_char, start.fence_len).is_ok()
    } else {
        false
    };

    // 내용 추출 (들여쓰기 제거)
    let content_lines: Vec<&str> = if has_closing_fence {
        lines[1..lines.len() - 1].to_vec()
    } else {
        lines[1..].to_vec()
    };

    let content = content_lines
        .iter()
        .map(|line| remove_indent(line, start.indent))
        .collect::<Vec<_>>()
        .join("\n");

    Some(Node::CodeBlock { info: start.info, content })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // === try_start 테스트 ===
    // expected: Ok((fence_char, fence_len, info, indent)) 또는 Err(reason)
    #[rstest]
    // 백틱 펜스
    #[case("```", Ok(('`', 3, None, 0)))]
    #[case("````", Ok(('`', 4, None, 0)))]
    #[case("`````", Ok(('`', 5, None, 0)))]
    // 틸드 펜스
    #[case("~~~", Ok(('~', 3, None, 0)))]
    #[case("~~~~", Ok(('~', 4, None, 0)))]
    #[case("~~~~~", Ok(('~', 5, None, 0)))]
    // info string
    #[case("```rust", Ok(('`', 3, Some("rust"), 0)))]
    #[case("~~~ python", Ok(('~', 3, Some("python"), 0)))]
    #[case("```  rust  ", Ok(('`', 3, Some("rust"), 0)))]  // 앞뒤 공백 제거
    #[case("```rust python", Ok(('`', 3, Some("rust python"), 0)))]  // 공백 포함
    // 들여쓰기 0-3칸
    #[case(" ```", Ok(('`', 3, None, 1)))]
    #[case("  ```", Ok(('`', 3, None, 2)))]
    #[case("   ```", Ok(('`', 3, None, 3)))]
    #[case("   ```rust", Ok(('`', 3, Some("rust"), 3)))]
    // 펜스가 아닌 경우
    #[case("``", Err(FencedCodeBlockNotStartReason::NoFence))]           // 백틱 2개
    #[case("~~", Err(FencedCodeBlockNotStartReason::NoFence))]           // 틸드 2개
    #[case("    ```", Err(FencedCodeBlockNotStartReason::IndentedCodeBlock))]  // 4칸 들여쓰기
    #[case("code", Err(FencedCodeBlockNotStartReason::NoFence))]         // 일반 텍스트
    #[case("", Err(FencedCodeBlockNotStartReason::NoFence))]             // 빈 줄
    #[case("  ", Err(FencedCodeBlockNotStartReason::NoFence))]           // 공백만
    fn test_try_start(
        #[case] input: &str,
        #[case] expected: Result<(char, usize, Option<&str>, usize), FencedCodeBlockNotStartReason>,
    ) {
        let result = try_start(input);
        match expected {
            Ok((expected_char, expected_len, expected_info, expected_indent)) => {
                assert!(result.is_ok(), "시작이어야 함: {:?}", input);
                let FencedCodeBlockStartReason::Started(start) = result.unwrap();
                assert_eq!(start.fence_char, expected_char, "fence_char");
                assert_eq!(start.fence_len, expected_len, "fence_len");
                assert_eq!(start.info.as_deref(), expected_info, "info");
                assert_eq!(start.indent, expected_indent, "indent");
            }
            Err(expected_reason) => {
                assert!(result.is_err(), "시작이 아니어야 함: {:?}", input);
                assert_eq!(result.unwrap_err(), expected_reason);
            }
        }
    }

    // === try_end 테스트 ===
    // expected: Ok(()) 또는 Err(reason)
    #[rstest]
    // 유효한 닫는 펜스
    #[case("```", '`', 3, Ok(()))]
    #[case("````", '`', 3, Ok(()))]      // 더 긴 펜스 OK
    #[case("`````", '`', 3, Ok(()))]
    #[case("~~~", '~', 3, Ok(()))]
    #[case("~~~~", '~', 3, Ok(()))]
    // 들여쓰기 0-3칸
    #[case(" ```", '`', 3, Ok(()))]
    #[case("  ```", '`', 3, Ok(()))]
    #[case("   ```", '`', 3, Ok(()))]
    // 펜스 뒤 공백만 허용
    #[case("```  ", '`', 3, Ok(()))]
    #[case("~~~   ", '~', 3, Ok(()))]
    // 유효하지 않은 닫는 펜스
    #[case("``", '`', 3, Err(FencedCodeBlockContinueReason::FenceTooShort))]       // 길이 부족
    #[case("```", '`', 4, Err(FencedCodeBlockContinueReason::FenceTooShort))]      // 최소 길이보다 짧음
    #[case("~~~", '`', 3, Err(FencedCodeBlockContinueReason::FenceCharMismatch))]  // 문자 불일치
    #[case("```", '~', 3, Err(FencedCodeBlockContinueReason::FenceCharMismatch))]  // 문자 불일치
    #[case("    ```", '`', 3, Err(FencedCodeBlockContinueReason::TooMuchIndent))]  // 4칸 들여쓰기
    #[case("```code", '`', 3, Err(FencedCodeBlockContinueReason::TextAfterFence))] // 펜스 뒤 텍스트
    #[case("``` x", '`', 3, Err(FencedCodeBlockContinueReason::TextAfterFence))]   // 펜스 뒤 텍스트
    fn test_try_end(
        #[case] input: &str,
        #[case] fence_char: char,
        #[case] min_len: usize,
        #[case] expected: Result<(), FencedCodeBlockContinueReason>,
    ) {
        let result = try_end(input, fence_char, min_len);
        match expected {
            Ok(()) => {
                assert!(result.is_ok(), "종료여야 함: {:?}", input);
            }
            Err(expected_reason) => {
                assert!(result.is_err(), "계속이어야 함: {:?}", input);
                assert_eq!(result.unwrap_err(), expected_reason);
            }
        }
    }

    // === parse 테스트 ===
    // expected: Some((content, info)) 또는 None (파싱 실패)
    #[rstest]
    #[case("```\ncode\n```", Some(("code", None)))]
    #[case("```\nline1\nline2\n```", Some(("line1\nline2", None)))]
    #[case("```\n\n```", Some(("", None)))]
    #[case("``\ncode\n``", None)]  // 백틱 2개는 펜스 아님
    #[case("code", None)]          // 펜스 없음
    // info string 테스트
    #[case("```rust\ncode\n```", Some(("code", Some("rust"))))]
    #[case("``` rust \ncode\n```", Some(("code", Some("rust"))))]  // 앞뒤 공백 제거
    #[case("```rust python\ncode\n```", Some(("code", Some("rust python"))))]  // 공백 포함
    // 틸드 펜스 테스트
    #[case("~~~\ncode\n~~~", Some(("code", None)))]
    #[case("~~~rust\ncode\n~~~", Some(("code", Some("rust"))))]
    #[case("~~\ncode\n~~", None)]  // 틸드 2개는 펜스 아님
    // 펜스 길이 매칭 테스트
    #[case("`````\ncode\n`````", Some(("code", None)))]  // 5개 == 5개
    #[case("```\ncode\n`````", Some(("code", None)))]    // 3개 < 5개 (닫는게 더 김)
    #[case("~~~~~\ncode\n~~~~~", Some(("code", None)))]  // 틸드도 동일
    // 유효하지 않은 닫는 펜스 → EOF까지 코드 (닫는 펜스도 내용에 포함)
    #[case("~~~\ncode\n```", Some(("code\n```", None)))]     // 문자 불일치
    #[case("```\ncode\n~~~", Some(("code\n~~~", None)))]     // 문자 불일치
    #[case("`````\ncode\n```", Some(("code\n```", None)))]   // 길이 부족
    #[case("~~~~~\ncode\n~~~", Some(("code\n~~~", None)))]   // 길이 부족
    // 닫는 펜스 없음 (EOF까지)
    #[case("```\ncode", Some(("code", None)))]
    #[case("```rust\ncode", Some(("code", Some("rust"))))]
    #[case("```\nline1\nline2", Some(("line1\nline2", None)))]
    #[case("~~~\ncode", Some(("code", None)))]
    // 들여쓰기 처리
    #[case("  ```\n  code\n  ```", Some(("code", None)))]        // 2칸 들여쓰기 제거
    #[case("   ```\n   code\n   ```", Some(("code", None)))]     // 3칸 들여쓰기 제거
    #[case("  ```\n    code\n  ```", Some(("  code", None)))]    // 2칸만 제거, 추가 2칸 유지
    #[case("  ```\ncode\n  ```", Some(("code", None)))]          // 내용에 들여쓰기 없어도 OK
    #[case("    ```\ncode\n```", None)]                          // 4칸 들여쓰기는 펜스 아님
    fn fenced_code_block(#[case] input: &str, #[case] expected: Option<(&str, Option<&str>)>) {
        let result = parse(input, 0);

        match expected {
            Some((content, info)) => {
                assert!(result.is_some(), "파싱 실패: {}", input);
                let node = result.unwrap();
                assert!(node.is_code_block(), "CodeBlock이 아님");
                assert_eq!(node.content(), content);
                assert_eq!(node.info(), info);
            }
            None => {
                assert!(result.is_none(), "펜스가 아닌데 파싱됨: {}", input);
            }
        }
    }
}
