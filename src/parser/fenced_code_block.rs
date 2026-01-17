//! Fenced Code Block 파서
//!
//! ``` 또는 ~~~로 감싸진 코드 블록을 파싱합니다.

use crate::node::Node;

/// Fenced Code Block 파싱 시도
/// 성공하면 Some(CodeBlock), 실패하면 None 반환
pub fn parse(text: &str, _indent: usize) -> Option<Node> {
    let lines: Vec<&str> = text.lines().collect();

    // 최소 2줄 필요 (여는 펜스 + 닫는 펜스)
    if lines.len() < 2 {
        return None;
    }

    // 여는 펜스 확인: ```로 시작
    let first_line = lines[0];
    if !first_line.starts_with("```") {
        return None;
    }

    // 닫는 펜스 확인: ```로 시작
    let last_line = lines[lines.len() - 1];
    if !last_line.starts_with("```") {
        return None;
    }

    // 중간 내용 추출
    let content = if lines.len() > 2 {
        lines[1..lines.len() - 1].join("\n")
    } else {
        String::new()
    };

    Some(Node::CodeBlock {
        info: None,
        content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("```\ncode\n```", Some("code"))]
    #[case("```\nline1\nline2\n```", Some("line1\nline2"))]
    #[case("```\n\n```", Some(""))]
    #[case("``\ncode\n``", None)]  // 백틱 2개는 펜스 아님
    #[case("code", None)]          // 펜스 없음
    fn fenced_code_block(#[case] input: &str, #[case] expected: Option<&str>) {
        let result = parse(input, 0);

        match expected {
            Some(content) => {
                assert!(result.is_some(), "파싱 실패: {}", input);
                let node = result.unwrap();
                assert!(node.is_code_block(), "CodeBlock이 아님");
                assert_eq!(node.content(), content);
                assert_eq!(node.info(), None);
            }
            None => {
                assert!(result.is_none(), "펜스가 아닌데 파싱됨: {}", input);
            }
        }
    }
}
