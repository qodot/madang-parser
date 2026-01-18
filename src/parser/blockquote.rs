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

    // \n\n으로 분리하여 각 블록 파싱
    let children: Vec<Node> = content
        .split("\n\n")
        .filter(|s| !s.is_empty())
        .map(|block| parse_block(block))
        .collect();

    Some(Node::Blockquote { children })
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

    /// 4칸 이상 들여쓰기는 Blockquote 아님 (code block)
    #[rstest]
    // Example 231: 4칸 들여쓰기는 code block
    #[case("    > # Foo")]
    fn test_not_blockquote(#[case] input: &str) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1);
        assert!(doc.children()[0].is_code_block(), "입력: {}", input);
    }

    /// Example 244: Blockquote 내 복수 단락 테스트
    #[rstest]
    #[case("> foo\n>\n> bar", &["foo", "bar"])]
    #[case("> line1\n>\n> line2", &["line1", "line2"])]
    #[case("> a\n>\n> b\n>\n> c", &["a", "b", "c"])]
    fn test_blockquote_multiple_paragraphs(#[case] input: &str, #[case] texts: &[&str]) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "입력: {}", input);

        let blockquote = &doc.children()[0];
        assert!(blockquote.is_blockquote(), "입력: {}", input);
        assert_eq!(blockquote.children().len(), texts.len(), "단락 수 불일치, 입력: {}", input);

        for (i, text) in texts.iter().enumerate() {
            let para = &blockquote.children()[i];
            assert_eq!(para.children()[0].as_text(), *text, "단락 {}, 입력: {}", i, input);
        }
    }

    /// 복수 블록 조합 테스트
    /// is_blockquote: 각 자식이 blockquote인지 여부
    #[rstest]
    // Example 242: 빈 줄로 분리된 두 blockquote
    #[case("> foo\n\n> bar", &[true, true])]
    // Example 245: Paragraph 후 blockquote
    #[case("foo\n> bar", &[false, true])]
    // Example 248: Blockquote 후 빈 줄 + paragraph
    #[case("> bar\n\nbaz", &[true, false])]
    fn test_block_sequence(#[case] input: &str, #[case] is_blockquote: &[bool]) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), is_blockquote.len(), "입력: {}", input);
        for (i, expected_bq) in is_blockquote.iter().enumerate() {
            assert_eq!(
                doc.children()[i].is_blockquote(),
                *expected_bq,
                "자식 {}, 입력: {}",
                i,
                input
            );
        }
    }

    /// CommonMark Block Quote 예제 테스트
    /// depth = None이면 Paragraph, Some(n)이면 n단계 중첩
    #[rstest]
    // Example 229: > 뒤 공백 없어도 OK
    #[case(">hello", Some(1), "hello")]
    #[case(">bar", Some(1), "bar")]
    // Example 230: 1-3칸 들여쓰기 허용
    #[case(" > hello", Some(1), "hello")]
    #[case("  > hello", Some(1), "hello")]
    #[case("   > hello", Some(1), "hello")]
    // Example 232-233: Lazy continuation
    #[case("> bar\nbaz", Some(1), "bar\nbaz")]
    #[case("> bar\nbaz\n> foo", Some(1), "bar\nbaz\nfoo")]
    // Example 243: 여러 줄 하나의 paragraph
    #[case("> foo\n> bar", Some(1), "foo\nbar")]
    // Example 247: Lazy continuation
    #[case("> bar\nbaz", Some(1), "bar\nbaz")]
    // Example 250: 중첩 blockquote + lazy continuation
    #[case("> > > foo\nbar", Some(3), "foo\nbar")]
    // 추가 케이스
    #[case("> hello", Some(1), "hello")]
    #[case(">  hello", Some(1), "hello")]
    #[case("> 안녕하세요", Some(1), "안녕하세요")]
    #[case("> > nested", Some(2), "nested")]
    #[case("> > > deep", Some(3), "deep")]
    #[case("> > > > 4단계", Some(4), "4단계")]
    #[case("> a\n> b\n> c", Some(1), "a\nb\nc")]
    #[case(">line1\n>line2", Some(1), "line1\nline2")]
    #[case("> > a\n> > b", Some(2), "a\nb")]
    #[case("> a\nb\nc", Some(1), "a\nb\nc")]
    #[case("> start\n> middle\nend", Some(1), "start\nmiddle\nend")]
    fn test_blockquote(#[case] input: &str, #[case] depth: Option<usize>, #[case] text: &str) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "입력: {}", input);

        match depth {
            Some(d) => {
                let mut current = &doc.children()[0];
                for i in 0..d {
                    assert!(current.is_blockquote(), "깊이 {}는 Blockquote여야 함, 입력: {}", i + 1, input);
                    if i < d - 1 {
                        current = &current.children()[0];
                    }
                }
                let para = &current.children()[0];
                assert_eq!(para.children()[0].as_text(), text, "입력: {}", input);
            }
            None => {
                assert!(!doc.children()[0].is_blockquote(), "Blockquote가 아니어야 함, 입력: {}", input);
                assert_eq!(doc.children()[0].children()[0].as_text(), text, "입력: {}", input);
            }
        }
    }

    /// Blockquote 내 다른 블록 요소 테스트
    /// heading = Some((level, text)), is_hr = true면 ThematicBreak
    #[rstest]
    // Example 228: Blockquote 내 heading
    #[case("> # Foo", Some((1, "Foo")), false)]
    #[case("> # Title", Some((1, "Title")), false)]
    #[case("> ## Subtitle", Some((2, "Subtitle")), false)]
    #[case("> ---", None, true)]
    #[case("> ***", None, true)]
    fn test_blockquote_inner_blocks(
        #[case] input: &str,
        #[case] heading: Option<(u8, &str)>,
        #[case] is_hr: bool,
    ) {
        let doc = parse(input);
        let blockquote = &doc.children()[0];
        assert!(blockquote.is_blockquote(), "입력: {}", input);

        let inner = &blockquote.children()[0];
        if let Some((level, text)) = heading {
            assert_eq!(inner.level(), level, "입력: {}", input);
            assert_eq!(inner.children()[0].as_text(), text, "입력: {}", input);
        }
        if is_hr {
            assert!(inner.is_thematic_break(), "입력: {}", input);
        }
    }
}
