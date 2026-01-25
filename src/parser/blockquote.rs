//! Blockquote 파싱
//!
//! CommonMark 명세: https://spec.commonmark.org/0.31.2/#block-quotes

use super::helpers::calculate_indent;
use crate::node::{BlockNode, BlockquoteNode};

#[derive(Debug, Clone, PartialEq)]
pub enum BlockquoteErr {
    /// 4칸 이상 들여쓰기 (코드 블록으로 해석됨)
    CodeBlockIndented,
    /// >로 시작하지 않음
    NotBlockquoteMarker,
}

/// Blockquote 라인 파싱 - 마커 검증 및 내용 추출
/// 성공하면 Ok(마커 제거된 내용), 실패하면 Err(사유)
pub fn parse(line: &str) -> Result<String, BlockquoteErr> {
    let indent = calculate_indent(line);
    let trimmed = line.trim();

    // 들여쓰기 3칸 초과면 Blockquote 아님
    if indent > 3 {
        return Err(BlockquoteErr::CodeBlockIndented);
    }

    // >로 시작하지 않으면 Blockquote 아님
    if !trimmed.starts_with('>') {
        return Err(BlockquoteErr::NotBlockquoteMarker);
    }

    // > 마커 제거 후 내용 반환
    let rest = &trimmed[1..];
    let content = if rest.starts_with(' ') || rest.starts_with('\t') {
        &rest[1..]
    } else {
        rest
    };

    Ok(content.to_string())
}

/// 축적된 내용으로 Blockquote 노드 생성
/// contents: 각 라인에서 > 마커 제거된 내용들
pub fn finalize<F>(contents: Vec<String>, parse_block: F) -> BlockNode
where
    F: Fn(&str) -> BlockNode,
{
    let text = contents.join("\n");

    // \n\n으로 분리하여 각 블록 파싱
    let children: Vec<BlockNode> = text
        .split("\n\n")
        .filter(|s| !s.is_empty())
        .map(parse_block)
        .collect();

    BlockNode::Blockquote(BlockquoteNode::new(children))
}

/// 여러 줄 텍스트를 Blockquote로 파싱 (중첩 blockquote용)
/// 첫 줄이 blockquote가 아니면 None 반환
pub fn parse_text<F>(text: &str, parse_block: F) -> Option<BlockNode>
where
    F: Fn(&str) -> BlockNode,
{
    let mut contents: Vec<String> = Vec::new();

    for line in text.lines() {
        match parse(line) {
            Ok(content) => contents.push(content),
            Err(BlockquoteErr::NotBlockquoteMarker) => {
                // Lazy continuation: > 없는 줄은 그대로 유지
                if contents.is_empty() {
                    // 첫 줄이 blockquote가 아니면 None
                    return None;
                }
                contents.push(line.trim().to_string());
            }
            Err(BlockquoteErr::CodeBlockIndented) => {
                // 4칸 이상 들여쓰기면 blockquote 아님
                if contents.is_empty() {
                    return None;
                }
                // 이미 시작된 blockquote 안에서는 lazy continuation으로 처리
                contents.push(line.trim().to_string());
            }
        }
    }

    if contents.is_empty() {
        return None;
    }

    Some(finalize(contents, parse_block))
}

#[cfg(test)]
mod tests {
    use crate::node::{BlockNode, InlineNode};
    use crate::parser::parse;
    use rstest::rstest;

    /// 중첩 blockquote 헬퍼 함수
    fn bq(depth: usize, inner: BlockNode) -> BlockNode {
        let mut result = inner;
        for _ in 0..depth {
            result = BlockNode::blockquote(vec![result]);
        }
        result
    }

    #[rstest]
    // Example 231: 4칸 들여쓰기는 code block
    #[case("    > # Foo", vec![BlockNode::code_block(None, "> # Foo")])]
    // Example 228: Blockquote 내 heading
    #[case("> # Foo", vec![BlockNode::blockquote(vec![BlockNode::heading(1, vec![InlineNode::text("Foo")])])])]
    #[case("> # Title", vec![BlockNode::blockquote(vec![BlockNode::heading(1, vec![InlineNode::text("Title")])])])]
    #[case("> ## Subtitle", vec![BlockNode::blockquote(vec![BlockNode::heading(2, vec![InlineNode::text("Subtitle")])])])]
    // Example 228: Blockquote 내 thematic break
    #[case("> ---", vec![BlockNode::blockquote(vec![BlockNode::thematic_break()])])]
    #[case("> ***", vec![BlockNode::blockquote(vec![BlockNode::thematic_break()])])]
    // Example 229: > 뒤 공백 없어도 OK
    #[case(">hello", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("hello")])])])]
    #[case(">bar", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("bar")])])])]
    // Example 230: 1-3칸 들여쓰기 허용
    #[case(" > hello", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("hello")])])])]
    #[case("  > hello", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("hello")])])])]
    #[case("   > hello", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("hello")])])])]
    // Example 232-233: Lazy continuation
    #[case("> bar\nbaz", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("bar\nbaz")])])])]
    #[case("> bar\nbaz\n> foo", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("bar\nbaz\nfoo")])])])]
    // Example 242: 빈 줄로 분리된 두 blockquote
    #[case("> foo\n\n> bar", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("foo")])]), BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("bar")])])])]
    // Example 243: 여러 줄 하나의 paragraph
    #[case("> foo\n> bar", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("foo\nbar")])])])]
    // Example 244: Blockquote 내 복수 단락
    #[case("> foo\n>\n> bar", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("foo")]), BlockNode::paragraph(vec![InlineNode::text("bar")])])])]
    #[case("> line1\n>\n> line2", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("line1")]), BlockNode::paragraph(vec![InlineNode::text("line2")])])])]
    #[case("> a\n>\n> b\n>\n> c", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("a")]), BlockNode::paragraph(vec![InlineNode::text("b")]), BlockNode::paragraph(vec![InlineNode::text("c")])])])]
    // Example 245: Paragraph 후 blockquote
    #[case("foo\n> bar", vec![BlockNode::paragraph(vec![InlineNode::text("foo")]), BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("bar")])])])]
    // Example 247: Lazy continuation
    #[case("> bar\nbaz", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("bar\nbaz")])])])]
    // Example 248: Blockquote 후 빈 줄 + paragraph
    #[case("> bar\n\nbaz", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("bar")])]), BlockNode::paragraph(vec![InlineNode::text("baz")])])]
    // 추가 케이스
    #[case("> hello", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("hello")])])])]
    #[case(">  hello", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("hello")])])])]
    #[case("> 안녕하세요", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("안녕하세요")])])])]
    #[case("> a\n> b\n> c", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("a\nb\nc")])])])]
    #[case(">line1\n>line2", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("line1\nline2")])])])]
    #[case("> a\nb\nc", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("a\nb\nc")])])])]
    #[case("> start\n> middle\nend", vec![BlockNode::blockquote(vec![BlockNode::paragraph(vec![InlineNode::text("start\nmiddle\nend")])])])]
    fn test_blockquote(#[case] input: &str, #[case] expected: Vec<BlockNode>) {
        let doc = parse(input);
        assert_eq!(doc.children, expected);
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
        let expected = vec![bq(depth, BlockNode::paragraph(vec![InlineNode::text(text)]))];
        assert_eq!(doc.children, expected);
    }
}
