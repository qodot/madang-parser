//! Fenced Code Block 파서
//!
//! ``` 또는 ~~~로 감싸진 코드 블록을 파싱합니다.

use crate::node::Node;

/// 문자열 앞에서 특정 문자가 연속으로 몇 개 있는지 세기
fn count_leading_char(s: &str, c: char) -> usize {
    s.chars().take_while(|&ch| ch == c).count()
}

/// Fenced Code Block 파싱 시도
/// 성공하면 Some(CodeBlock), 실패하면 None 반환
pub fn parse(text: &str, _indent: usize) -> Option<Node> {
    let lines: Vec<&str> = text.lines().collect();

    if lines.is_empty() {
        return None;
    }

    // 여는 펜스 확인: ``` 또는 ~~~로 시작
    let first_line = lines[0];
    let (fence_char, fence_len) = if first_line.starts_with("```") {
        ('`', count_leading_char(first_line, '`'))
    } else if first_line.starts_with("~~~") {
        ('~', count_leading_char(first_line, '~'))
    } else {
        return None;
    };

    // info string 추출: 펜스 마커 이후 문자열
    let info = {
        let after_fence = &first_line[fence_len..];
        let trimmed = after_fence.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    };

    // 닫는 펜스 찾기: 같은 문자로 시작, 같거나 더 긴 길이
    let has_closing_fence = if lines.len() >= 2 {
        let last_line = lines[lines.len() - 1];
        let closing_len = count_leading_char(last_line, fence_char);
        closing_len >= fence_len
    } else {
        false
    };

    // 내용 추출
    let content = if has_closing_fence {
        // 닫는 펜스가 있으면 마지막 줄 제외
        if lines.len() > 2 {
            lines[1..lines.len() - 1].join("\n")
        } else {
            String::new()
        }
    } else {
        // 닫는 펜스가 없으면 첫 줄 이후 전체
        if lines.len() > 1 {
            lines[1..].join("\n")
        } else {
            String::new()
        }
    };

    Some(Node::CodeBlock { info, content })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

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
