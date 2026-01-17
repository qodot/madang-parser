//! Fenced Code Block 파서
//!
//! 백틱(\`\`\`) 또는 틸드(~~~)로 감싸진 코드 블록을 파싱합니다.

use crate::node::Node;
use super::helpers::{count_leading_char, remove_indent};

/// Fenced Code Block 시작 줄인지 확인
/// 반환: (fence_char, fence_len, info, indent)
pub(crate) fn try_start(line: &str) -> Option<(char, usize, Option<String>, usize)> {
    let indent = count_leading_char(line, ' ');
    if indent > 3 {
        return None;
    }

    let after_indent = &line[indent..];

    let (fence_char, fence_len) = if after_indent.starts_with("```") {
        ('`', count_leading_char(after_indent, '`'))
    } else if after_indent.starts_with("~~~") {
        ('~', count_leading_char(after_indent, '~'))
    } else {
        return None;
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

    Some((fence_char, fence_len, info, indent))
}

/// 닫는 펜스인지 확인
pub(crate) fn is_end(line: &str, fence_char: char, min_fence_len: usize) -> bool {
    let indent = count_leading_char(line, ' ');
    if indent > 3 {
        return false;
    }

    let after_indent = &line[indent..];
    let closing_len = count_leading_char(after_indent, fence_char);

    if closing_len < min_fence_len {
        return false;
    }

    after_indent[closing_len..].trim().is_empty()
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
    let (fence_char, fence_len, info, opening_indent) = try_start(first_line)?;

    // 닫는 펜스 찾기
    let has_closing_fence = if lines.len() >= 2 {
        let last_line = lines[lines.len() - 1];
        is_end(last_line, fence_char, fence_len)
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
        .map(|line| remove_indent(line, opening_indent))
        .collect::<Vec<_>>()
        .join("\n");

    Some(Node::CodeBlock { info, content })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // === try_start 테스트 ===
    // expected: Some((fence_char, fence_len, info, indent)) 또는 None
    #[rstest]
    // 백틱 펜스
    #[case("```", Some(('`', 3, None, 0)))]
    #[case("````", Some(('`', 4, None, 0)))]
    #[case("`````", Some(('`', 5, None, 0)))]
    // 틸드 펜스
    #[case("~~~", Some(('~', 3, None, 0)))]
    #[case("~~~~", Some(('~', 4, None, 0)))]
    #[case("~~~~~", Some(('~', 5, None, 0)))]
    // info string
    #[case("```rust", Some(('`', 3, Some("rust"), 0)))]
    #[case("~~~ python", Some(('~', 3, Some("python"), 0)))]
    #[case("```  rust  ", Some(('`', 3, Some("rust"), 0)))]  // 앞뒤 공백 제거
    #[case("```rust python", Some(('`', 3, Some("rust python"), 0)))]  // 공백 포함
    // 들여쓰기 0-3칸
    #[case(" ```", Some(('`', 3, None, 1)))]
    #[case("  ```", Some(('`', 3, None, 2)))]
    #[case("   ```", Some(('`', 3, None, 3)))]
    #[case("   ```rust", Some(('`', 3, Some("rust"), 3)))]
    // 펜스가 아닌 경우
    #[case("``", None)]           // 백틱 2개
    #[case("~~", None)]           // 틸드 2개
    #[case("    ```", None)]      // 4칸 들여쓰기
    #[case("code", None)]         // 일반 텍스트
    #[case("", None)]             // 빈 줄
    #[case("  ", None)]           // 공백만
    fn test_try_start(#[case] input: &str, #[case] expected: Option<(char, usize, Option<&str>, usize)>) {
        let result = try_start(input);
        match expected {
            Some((char, len, info, indent)) => {
                assert!(result.is_some(), "시작이어야 함: {:?}", input);
                let (c, l, i, ind) = result.unwrap();
                assert_eq!(c, char, "fence_char");
                assert_eq!(l, len, "fence_len");
                assert_eq!(i.as_deref(), info, "info");
                assert_eq!(ind, indent, "indent");
            }
            None => {
                assert!(result.is_none(), "시작이 아니어야 함: {:?}", input);
            }
        }
    }

    // === is_end 테스트 ===
    #[rstest]
    // 유효한 닫는 펜스
    #[case("```", '`', 3, true)]
    #[case("````", '`', 3, true)]      // 더 긴 펜스 OK
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
    // 유효하지 않은 닫는 펜스
    #[case("``", '`', 3, false)]       // 길이 부족
    #[case("```", '`', 4, false)]      // 최소 길이보다 짧음
    #[case("~~~", '`', 3, false)]      // 문자 불일치
    #[case("```", '~', 3, false)]      // 문자 불일치
    #[case("    ```", '`', 3, false)]  // 4칸 들여쓰기
    #[case("```code", '`', 3, false)]  // 펜스 뒤 텍스트
    #[case("``` x", '`', 3, false)]    // 펜스 뒤 텍스트
    fn test_is_end(#[case] input: &str, #[case] fence_char: char, #[case] min_len: usize, #[case] expected: bool) {
        assert_eq!(is_end(input, fence_char, min_len), expected, "input: {:?}", input);
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
