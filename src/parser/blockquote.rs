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

    // === CommonMark Block Quote 예제 테스트 ===
    // depth = None이면 Paragraph, Some(n)이면 n단계 중첩
    #[rstest]
    // === Example 229: > 뒤 공백 없어도 OK ===
    #[case(">hello", Some(1), "hello")]
    #[case(">bar", Some(1), "bar")]
    // === Example 230: 1-3칸 들여쓰기 허용 ===
    #[case(" > hello", Some(1), "hello")]
    #[case("  > hello", Some(1), "hello")]
    #[case("   > hello", Some(1), "hello")]
    // === Example 232-233: Lazy continuation ===
    #[case("> bar\nbaz", Some(1), "bar\nbaz")]
    #[case("> bar\nbaz\n> foo", Some(1), "bar\nbaz\nfoo")]
    // === Example 243: 여러 줄 하나의 paragraph ===
    #[case("> foo\n> bar", Some(1), "foo\nbar")]
    // === Example 247: Lazy continuation ===
    #[case("> bar\nbaz", Some(1), "bar\nbaz")]
    // === Example 250: 중첩 blockquote + lazy continuation ===
    #[case("> > > foo\nbar", Some(3), "foo\nbar")]
    // === 추가 케이스 ===
    #[case("> hello", Some(1), "hello")]
    #[case(">  hello", Some(1), "hello")]                 // 여러 공백
    #[case("> 안녕하세요", Some(1), "안녕하세요")]        // 유니코드
    #[case("> > nested", Some(2), "nested")]              // 2단계 중첩
    #[case("> > > deep", Some(3), "deep")]                // 3단계 중첩
    #[case("> > > > 4단계", Some(4), "4단계")]            // 4단계 중첩
    #[case("> a\n> b\n> c", Some(1), "a\nb\nc")]          // 연속 줄
    #[case(">line1\n>line2", Some(1), "line1\nline2")]    // 공백 없이
    #[case("> > a\n> > b", Some(2), "a\nb")]              // 중첩 다중줄
    #[case("> a\nb\nc", Some(1), "a\nb\nc")]              // 여러 줄 lazy
    #[case("> start\n> middle\nend", Some(1), "start\nmiddle\nend")]  // 혼합
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

    // === Example 244: Blockquote 내 복수 단락 테스트 ===
    #[rstest]
    #[case("> foo\n>\n> bar", &["foo", "bar"])]                      // Example 244
    #[case("> line1\n>\n> line2", &["line1", "line2"])]              // 빈 > 로 분리
    #[case("> a\n>\n> b\n>\n> c", &["a", "b", "c"])]                 // 3개 단락
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

    // === Example 228: Blockquote 내 heading (단독) ===
    // 참고: 현재 구현은 heading 뒤 paragraph를 별도 블록으로 분리하지 않음
    // 빈 줄 없이 연속된 내용은 하나의 블록으로 처리됨
    #[test]
    fn test_example_228_blockquote_with_heading() {
        let doc = parse("> # Foo");
        assert_eq!(doc.children().len(), 1, "하나의 blockquote");
        let blockquote = &doc.children()[0];
        assert!(blockquote.is_blockquote());
        // Heading이 첫 번째 자식
        assert_eq!(blockquote.children()[0].level(), 1);
        assert_eq!(blockquote.children()[0].children()[0].as_text(), "Foo");
    }

    // === Blockquote 내 다른 블록 요소 테스트 ===
    // heading = Some((level, text)), is_hr = true면 ThematicBreak
    #[rstest]
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

    // === Example 231: 4칸 들여쓰기는 code block ===
    #[test]
    fn test_example_231_four_space_indent() {
        let doc = parse("    > # Foo");
        assert_eq!(doc.children().len(), 1);
        assert!(doc.children()[0].is_code_block(), "4칸 들여쓰기는 CodeBlock");
    }

    // === Example 242: 빈 줄로 분리된 두 blockquote ===
    #[test]
    fn test_example_242_separate_blockquotes() {
        let doc = parse("> foo\n\n> bar");
        assert_eq!(doc.children().len(), 2, "두 개의 blockquote");
        assert!(doc.children()[0].is_blockquote());
        assert!(doc.children()[1].is_blockquote());
    }

    // === Example 245: Paragraph 후 blockquote ===
    #[test]
    fn test_example_245_paragraph_then_blockquote() {
        let doc = parse("foo\n> bar");
        assert_eq!(doc.children().len(), 2, "paragraph + blockquote");
        // 첫 번째는 Paragraph
        assert!(!doc.children()[0].is_blockquote());
        // 두 번째는 Blockquote
        assert!(doc.children()[1].is_blockquote());
    }

    // === Example 248: Blockquote 후 빈 줄 + paragraph ===
    #[test]
    fn test_example_248_blockquote_then_paragraph() {
        let doc = parse("> bar\n\nbaz");
        assert_eq!(doc.children().len(), 2, "blockquote + paragraph");
        assert!(doc.children()[0].is_blockquote());
        assert!(!doc.children()[1].is_blockquote());
    }
}
