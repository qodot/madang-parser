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
    use crate::node::Node;
    use crate::parser::parse;
    use rstest::rstest;

    /// 중첩 blockquote 헬퍼 함수
    fn bq(depth: usize, inner: Node) -> Node {
        let mut result = inner;
        for _ in 0..depth {
            result = Node::blockquote(vec![result]);
        }
        result
    }

    #[rstest]
    // Example 231: 4칸 들여쓰기는 code block
    #[case("    > # Foo", vec![Node::code_block(None, "> # Foo")])]
    // Example 228: Blockquote 내 heading
    #[case("> # Foo", vec![Node::blockquote(vec![Node::heading(1, vec![Node::text("Foo")])])])]
    #[case("> # Title", vec![Node::blockquote(vec![Node::heading(1, vec![Node::text("Title")])])])]
    #[case("> ## Subtitle", vec![Node::blockquote(vec![Node::heading(2, vec![Node::text("Subtitle")])])])]
    // Example 228: Blockquote 내 thematic break
    #[case("> ---", vec![Node::blockquote(vec![Node::ThematicBreak])])]
    #[case("> ***", vec![Node::blockquote(vec![Node::ThematicBreak])])]
    // Example 229: > 뒤 공백 없어도 OK
    #[case(">hello", vec![Node::blockquote(vec![Node::para(vec![Node::text("hello")])])])]
    #[case(">bar", vec![Node::blockquote(vec![Node::para(vec![Node::text("bar")])])])]
    // Example 230: 1-3칸 들여쓰기 허용
    #[case(" > hello", vec![Node::blockquote(vec![Node::para(vec![Node::text("hello")])])])]
    #[case("  > hello", vec![Node::blockquote(vec![Node::para(vec![Node::text("hello")])])])]
    #[case("   > hello", vec![Node::blockquote(vec![Node::para(vec![Node::text("hello")])])])]
    // Example 232-233: Lazy continuation
    #[case("> bar\nbaz", vec![Node::blockquote(vec![Node::para(vec![Node::text("bar\nbaz")])])])]
    #[case("> bar\nbaz\n> foo", vec![Node::blockquote(vec![Node::para(vec![Node::text("bar\nbaz\nfoo")])])])]
    // Example 242: 빈 줄로 분리된 두 blockquote
    #[case("> foo\n\n> bar", vec![Node::blockquote(vec![Node::para(vec![Node::text("foo")])]), Node::blockquote(vec![Node::para(vec![Node::text("bar")])])])]
    // Example 243: 여러 줄 하나의 paragraph
    #[case("> foo\n> bar", vec![Node::blockquote(vec![Node::para(vec![Node::text("foo\nbar")])])])]
    // Example 244: Blockquote 내 복수 단락
    #[case("> foo\n>\n> bar", vec![Node::blockquote(vec![Node::para(vec![Node::text("foo")]), Node::para(vec![Node::text("bar")])])])]
    #[case("> line1\n>\n> line2", vec![Node::blockquote(vec![Node::para(vec![Node::text("line1")]), Node::para(vec![Node::text("line2")])])])]
    #[case("> a\n>\n> b\n>\n> c", vec![Node::blockquote(vec![Node::para(vec![Node::text("a")]), Node::para(vec![Node::text("b")]), Node::para(vec![Node::text("c")])])])]
    // Example 245: Paragraph 후 blockquote
    #[case("foo\n> bar", vec![Node::para(vec![Node::text("foo")]), Node::blockquote(vec![Node::para(vec![Node::text("bar")])])])]
    // Example 247: Lazy continuation
    #[case("> bar\nbaz", vec![Node::blockquote(vec![Node::para(vec![Node::text("bar\nbaz")])])])]
    // Example 248: Blockquote 후 빈 줄 + paragraph
    #[case("> bar\n\nbaz", vec![Node::blockquote(vec![Node::para(vec![Node::text("bar")])]), Node::para(vec![Node::text("baz")])])]
    // 추가 케이스
    #[case("> hello", vec![Node::blockquote(vec![Node::para(vec![Node::text("hello")])])])]
    #[case(">  hello", vec![Node::blockquote(vec![Node::para(vec![Node::text("hello")])])])]
    #[case("> 안녕하세요", vec![Node::blockquote(vec![Node::para(vec![Node::text("안녕하세요")])])])]
    #[case("> a\n> b\n> c", vec![Node::blockquote(vec![Node::para(vec![Node::text("a\nb\nc")])])])]
    #[case(">line1\n>line2", vec![Node::blockquote(vec![Node::para(vec![Node::text("line1\nline2")])])])]
    #[case("> a\nb\nc", vec![Node::blockquote(vec![Node::para(vec![Node::text("a\nb\nc")])])])]
    #[case("> start\n> middle\nend", vec![Node::blockquote(vec![Node::para(vec![Node::text("start\nmiddle\nend")])])])]
    fn test_blockquote(#[case] input: &str, #[case] expected: Vec<Node>) {
        let doc = parse(input);
        assert_eq!(doc.children(), &expected);
    }

    /// 중첩 blockquote 테스트
    #[rstest]
    // Example 250: 중첩 blockquote + lazy continuation
    #[case("> > > foo\nbar", 3, "foo\nbar")]
    // 추가 케이스
    #[case("> > nested", 2, "nested")]
    #[case("> > > deep", 3, "deep")]
    #[case("> > > > 4단계", 4, "4단계")]
    #[case("> > a\n> > b", 2, "a\nb")]
    fn test_nested_blockquote(#[case] input: &str, #[case] depth: usize, #[case] text: &str) {
        let doc = parse(input);
        let expected = vec![bq(depth, Node::para(vec![Node::text(text)]))];
        assert_eq!(doc.children(), &expected);
    }
}
