//! Blockquote 파싱
//!
//! CommonMark 명세: https://spec.commonmark.org/0.31.2/#block-quotes

use crate::node::Node;

/// Blockquote 파싱 시도
/// 성공하면 Some(Node::Blockquote), 실패하면 None
/// 중첩 blockquote를 위해 parse_block 함수를 받음
pub fn parse<F>(trimmed: &str, indent: usize, parse_block: F) -> Option<Node>
where
    F: Fn(&str) -> Node,
{
    // 들여쓰기 3칸 초과면 Blockquote 아님
    if indent > 3 {
        return None;
    }

    // >로 시작하지 않으면 Blockquote 아님
    if !trimmed.starts_with('>') {
        return None;
    }

    // 각 줄에서 > 마커 제거
    let content = strip_blockquote_markers(trimmed);

    // 재귀적으로 내용 파싱
    let inner = parse_block(&content);
    Some(Node::Blockquote {
        children: vec![inner],
    })
}

/// 각 줄에서 blockquote 마커(>) 제거
fn strip_blockquote_markers(text: &str) -> String {
    text.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with('>') {
                let rest = &trimmed[1..];
                // > 뒤 공백 하나 제거
                if rest.starts_with(' ') || rest.starts_with('\t') {
                    &rest[1..]
                } else {
                    rest
                }
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;
    use rstest::rstest;

    // depth = None이면 Paragraph, Some(n)이면 n단계 중첩
    #[rstest]
    // 단순 케이스 (depth = 1)
    #[case("> hello", Some(1), "hello")]
    #[case(">hello", Some(1), "hello")]                   // 공백 없어도 OK
    #[case(">  hello", Some(1), "hello")]                 // 여러 공백
    #[case(" > hello", Some(1), "hello")]                 // 선행 공백 1칸
    #[case("  > hello", Some(1), "hello")]                // 선행 공백 2칸
    #[case("   > hello", Some(1), "hello")]               // 선행 공백 3칸
    #[case("> 안녕하세요", Some(1), "안녕하세요")]        // 유니코드
    // 중첩 케이스
    #[case("> > nested", Some(2), "nested")]
    #[case("> > > deep", Some(3), "deep")]
    #[case("> > > > 4단계", Some(4), "4단계")]
    // Blockquote가 아닌 케이스
    #[case("    > hello", None, "> hello")]               // 4칸 들여쓰기 → Paragraph
    // 다중줄 케이스
    #[case("> line1\n> line2", Some(1), "line1\nline2")]       // 연속 줄
    #[case("> a\n> b\n> c", Some(1), "a\nb\nc")]               // 3줄
    #[case(">line1\n>line2", Some(1), "line1\nline2")]         // 공백 없이
    // 중첩 다중줄
    #[case("> > a\n> > b", Some(2), "a\nb")]                   // 2단계 중첩 다중줄
    fn test_blockquote(#[case] input: &str, #[case] depth: Option<usize>, #[case] text: &str) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "입력: {}", input);

        match depth {
            Some(d) => {
                // 중첩 깊이만큼 Blockquote 따라가기
                let mut current = &doc.children()[0];
                for i in 0..d {
                    assert!(current.is_blockquote(), "깊이 {}는 Blockquote여야 함, 입력: {}", i + 1, input);
                    if i < d - 1 {
                        current = &current.children()[0];
                    }
                }
                // 마지막 Blockquote 안의 Paragraph 확인
                let para = &current.children()[0];
                assert_eq!(para.children()[0].as_text(), text, "입력: {}", input);
            }
            None => {
                // Blockquote가 아닌 경우 → Paragraph
                assert!(!doc.children()[0].is_blockquote(), "Blockquote가 아니어야 함, 입력: {}", input);
                assert_eq!(doc.children()[0].children()[0].as_text(), text, "입력: {}", input);
            }
        }
    }
}
