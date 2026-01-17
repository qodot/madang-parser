//! CommonMark 파서
//!
//! 라인 단위로 스캔하며 블록 레벨 요소를 파싱합니다.
//! fold 패턴을 사용하여 불변 상태 전이를 구현합니다.

mod blockquote;
mod context;
mod fenced_code_block;
mod heading;
mod helpers;
mod list;
mod list_item;
mod paragraph;
mod thematic_break;

use crate::node::Node;
use context::{
    FencedCodeBlockStart, ListContinueReason, ListEndReason, ListItemStart, ParsingContext,
};
use fenced_code_block::{
    is_end as is_end_fenced_code_block, try_start as try_start_fenced_code_block,
};
use helpers::{calculate_indent, remove_indent};

/// 파서 상태: (완성된 노드들, 현재 컨텍스트)
type ParserState = (Vec<Node>, ParsingContext);

/// 문서 전체 파싱
pub fn parse(input: &str) -> Node {
    if input.is_empty() {
        return Node::Document { children: vec![] };
    }

    // fold: 각 줄을 처리하며 상태 전이
    let (children, final_context) = input.lines().fold(
        (Vec::new(), ParsingContext::None),
        |(children, context), line| process_line(line, context, children),
    );

    // 마지막 컨텍스트 마무리
    let children = finalize_context(final_context, children);

    Node::Document { children }
}

/// 한 줄 처리 후 새 상태 반환
fn process_line(line: &str, context: ParsingContext, children: Vec<Node>) -> ParserState {
    match context {
        ParsingContext::None => process_line_in_none(line, children),
        ParsingContext::FencedCodeBlock { start, content } => {
            process_line_in_code_block(line, start, content, children)
        }
        ParsingContext::Paragraph { lines } => process_line_in_paragraph(line, lines, children),
        ParsingContext::Blockquote { lines } => process_line_in_blockquote(line, lines, children),
        ParsingContext::List {
            first_item_start,
            items,
            current_item_lines,
            tight,
            pending_blank_count,
        } => process_line_in_list(line, first_item_start, items, current_item_lines, tight, pending_blank_count, children),
    }
}

/// None 상태에서 줄 처리: 새 블록 시작 감지
fn process_line_in_none(line: &str, children: Vec<Node>) -> ParserState {
    // 빈 줄은 무시
    if line.trim().is_empty() {
        return (children, ParsingContext::None);
    }

    // Fenced Code Block 시작 감지
    if let Some(start) = try_start_fenced_code_block(line) {
        let context = ParsingContext::FencedCodeBlock {
            start,
            content: Vec::new(),
        };
        return (children, context);
    }

    // 한 줄 블록들 시도 (Thematic Break, ATX Heading)
    let indent = calculate_indent(line);
    let trimmed = line.trim();

    if let Some(node) = thematic_break::parse(trimmed, indent) {
        let children = push_node(children, node);
        return (children, ParsingContext::None);
    }

    if let Some(node) = heading::parse(trimmed, indent) {
        let children = push_node(children, node);
        return (children, ParsingContext::None);
    }

    // Blockquote 시작 감지 (> 로 시작하고 들여쓰기 3칸 이하)
    if trimmed.starts_with('>') && indent <= 3 {
        let context = ParsingContext::Blockquote {
            lines: vec![trimmed.to_string()],
        };
        return (children, context);
    }

    // List 시작 감지
    if let Some(start) = list_item::try_start(line) {
        let content = start.content.clone();
        let context = ParsingContext::List {
            first_item_start: start,
            items: Vec::new(),
            current_item_lines: vec![content],
            tight: true,
            pending_blank_count: 0,
        };
        return (children, context);
    }

    // 나머지는 Paragraph 시작
    let context = ParsingContext::Paragraph {
        lines: vec![line.trim().to_string()],
    };
    (children, context)
}

/// Code Block 상태에서 줄 처리
fn process_line_in_code_block(
    line: &str,
    start: FencedCodeBlockStart,
    content: Vec<String>,
    children: Vec<Node>,
) -> ParserState {
    // 닫는 펜스인지 확인
    if is_end_fenced_code_block(line, start.fence_char, start.fence_len) {
        let content_str = content.join("\n");
        let node = Node::CodeBlock {
            info: start.info,
            content: content_str,
        };
        let children = push_node(children, node);
        return (children, ParsingContext::None);
    }

    // 코드 줄 추가
    let code_line = remove_indent(line, start.indent);
    let content = push_string(content, code_line.to_string());

    let context = ParsingContext::FencedCodeBlock { start, content };
    (children, context)
}

/// Paragraph 상태에서 줄 처리
fn process_line_in_paragraph(line: &str, lines: Vec<String>, children: Vec<Node>) -> ParserState {
    // 빈 줄이면 Paragraph 종료
    if line.trim().is_empty() {
        let text = lines.join("\n");
        let children = push_node(children, paragraph::parse(&text));
        return (children, ParsingContext::None);
    }

    // Fenced Code Block 시작이면 Paragraph 종료 후 Code Block 시작
    if let Some(start) = try_start_fenced_code_block(line) {
        let text = lines.join("\n");
        let children = push_node(children, paragraph::parse(&text));
        let context = ParsingContext::FencedCodeBlock {
            start,
            content: Vec::new(),
        };
        return (children, context);
    }

    // Thematic Break이면 Paragraph 종료
    let trimmed = line.trim();
    let indent = calculate_indent(line);
    if let Some(node) = thematic_break::parse(trimmed, indent) {
        let text = lines.join("\n");
        let children = push_node(children, paragraph::parse(&text));
        let children = push_node(children, node);
        return (children, ParsingContext::None);
    }

    // ATX Heading이면 Paragraph 종료
    if let Some(node) = heading::parse(trimmed, indent) {
        let text = lines.join("\n");
        let children = push_node(children, paragraph::parse(&text));
        let children = push_node(children, node);
        return (children, ParsingContext::None);
    }

    // Blockquote 시작이면 Paragraph 종료 후 Blockquote 시작
    if trimmed.starts_with('>') && indent <= 3 {
        let text = lines.join("\n");
        let children = push_node(children, paragraph::parse(&text));
        let context = ParsingContext::Blockquote {
            lines: vec![trimmed.to_string()],
        };
        return (children, context);
    }

    // 줄 추가
    let lines = push_string(lines, line.trim().to_string());
    (children, ParsingContext::Paragraph { lines })
}

/// List 상태에서 줄 처리
fn process_line_in_list(
    line: &str,
    first_item_start: ListItemStart,
    items: Vec<Vec<String>>,
    current_item_lines: Vec<String>,
    tight: bool,
    pending_blank_count: usize,
    children: Vec<Node>,
) -> ParserState {
    match list_item::try_end(line, &first_item_start.marker, first_item_start.content_indent) {
        // 종료
        Ok(ListEndReason::Reprocess) => {
            let children = finalize_list(&first_item_start, items, current_item_lines, tight, children);
            process_line_in_none(line, children)
        }
        // 계속
        Err(ListContinueReason::Blank) => {
            let context = ParsingContext::List {
                first_item_start,
                items,
                current_item_lines,
                tight,
                pending_blank_count: pending_blank_count + 1,
            };
            (children, context)
        }
        Err(ListContinueReason::NewItem(new_start)) => {
            let items = push_item(items, current_item_lines);
            // 빈 줄이 있었으면 loose list
            let tight = tight && pending_blank_count == 0;
            let context = ParsingContext::List {
                first_item_start,
                items,
                current_item_lines: vec![new_start.content],
                tight,
                pending_blank_count: 0,
            };
            (children, context)
        }
        Err(ListContinueReason::ContinuationLine(content)) => {
            // 대기 중인 빈 줄을 내용에 추가
            let mut lines = current_item_lines;
            for _ in 0..pending_blank_count {
                lines = push_string(lines, String::new());
            }
            let lines = push_string(lines, content);
            let context = ParsingContext::List {
                first_item_start,
                items,
                current_item_lines: lines,
                tight,
                pending_blank_count: 0,
            };
            (children, context)
        }
    }
}

/// List를 완성하여 children에 추가
fn finalize_list(
    first_item_start: &ListItemStart,
    items: Vec<Vec<String>>,
    current_item_lines: Vec<String>,
    tight: bool,
    children: Vec<Node>,
) -> Vec<Node> {
    let (list_type, start) = first_item_start.marker.to_list_type();
    let all_items = push_item(items, current_item_lines);
    let list_node = Node::build_list(list_type, start, tight, all_items, paragraph::parse);
    push_node(children, list_node)
}



/// 아이템 리스트에 아이템 추가
fn push_item(mut items: Vec<Vec<String>>, item: Vec<String>) -> Vec<Vec<String>> {
    items.push(item);
    items
}

/// Blockquote 상태에서 줄 처리
fn process_line_in_blockquote(line: &str, lines: Vec<String>, children: Vec<Node>) -> ParserState {
    let trimmed = line.trim();
    let indent = calculate_indent(line);

    // 빈 줄이면 Blockquote 종료
    if trimmed.is_empty() {
        let text = lines.join("\n");
        if let Some(node) = blockquote::parse(&text, 0, parse_block_simple) {
            let children = push_node(children, node);
            return (children, ParsingContext::None);
        }
        // blockquote 파싱 실패시 (이론상 발생 안함)
        return (children, ParsingContext::None);
    }

    // Fenced Code Block 시작이면 Blockquote 종료
    if let Some(start) = try_start_fenced_code_block(line) {
        let text = lines.join("\n");
        let children = if let Some(node) = blockquote::parse(&text, 0, parse_block_simple) {
            push_node(children, node)
        } else {
            children
        };
        let context = ParsingContext::FencedCodeBlock {
            start,
            content: Vec::new(),
        };
        return (children, context);
    }

    // Thematic Break이면 Blockquote 종료
    if let Some(node) = thematic_break::parse(trimmed, indent) {
        let text = lines.join("\n");
        if let Some(bq_node) = blockquote::parse(&text, 0, parse_block_simple) {
            let children = push_node(children, bq_node);
            let children = push_node(children, node);
            return (children, ParsingContext::None);
        }
        let children = push_node(children, node);
        return (children, ParsingContext::None);
    }

    // ATX Heading이면 Blockquote 종료
    if let Some(node) = heading::parse(trimmed, indent) {
        let text = lines.join("\n");
        if let Some(bq_node) = blockquote::parse(&text, 0, parse_block_simple) {
            let children = push_node(children, bq_node);
            let children = push_node(children, node);
            return (children, ParsingContext::None);
        }
        let children = push_node(children, node);
        return (children, ParsingContext::None);
    }

    // > 로 시작하거나 lazy continuation이면 Blockquote 계속
    // (> 로 시작하지 않아도 Blockquote 안에서는 줄이 계속됨)
    let lines = push_string(lines, trimmed.to_string());
    (children, ParsingContext::Blockquote { lines })
}

/// 마지막 컨텍스트 마무리
fn finalize_context(context: ParsingContext, children: Vec<Node>) -> Vec<Node> {
    match context {
        ParsingContext::None => children,
        ParsingContext::FencedCodeBlock { start, content } => {
            let content_str = content.join("\n");
            let node = Node::CodeBlock {
                info: start.info,
                content: content_str,
            };
            push_node(children, node)
        }
        ParsingContext::Paragraph { lines } => {
            let text = lines.join("\n");
            push_node(children, paragraph::parse(&text))
        }
        ParsingContext::Blockquote { lines } => {
            let text = lines.join("\n");
            if let Some(node) = blockquote::parse(&text, 0, parse_block_simple) {
                push_node(children, node)
            } else {
                children
            }
        }
        ParsingContext::List {
            first_item_start,
            items,
            current_item_lines,
            tight,
            pending_blank_count: _,
        } => finalize_list(&first_item_start, items, current_item_lines, tight, children)
    }
}

/// 벡터에 요소 추가 후 반환 (불변 스타일)
fn push_node(mut vec: Vec<Node>, node: Node) -> Vec<Node> {
    vec.push(node);
    vec
}

/// 문자열 벡터에 요소 추가 후 반환
fn push_string(mut vec: Vec<String>, s: String) -> Vec<String> {
    vec.push(s);
    vec
}

/// 단일 블록 파싱 (blockquote 내부 등에서 사용)
fn parse_block_simple(block: &str) -> Node {
    let indent = calculate_indent(block);
    let trimmed = block.trim();

    if let Some(node) = fenced_code_block::parse(block, indent) {
        return node;
    }

    // 중첩 blockquote 처리를 위해 blockquote 파싱 시도
    if let Some(node) = blockquote::parse(trimmed, indent, parse_block_simple) {
        return node;
    }

    thematic_break::parse(trimmed, indent)
        .or_else(|| heading::parse(trimmed, indent))
        .unwrap_or_else(|| paragraph::parse(trimmed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn parse_empty_string() {
        let doc = parse("");
        assert_eq!(doc.children().len(), 0);
    }

    /// 코드 블록 안 빈 줄 테스트
    #[rstest]
    #[case("```\nline1\n\nline2\n```", "line1\n\nline2")]
    #[case(
        "```rust\nfn main() {\n\n    println!(\"hi\");\n}\n```",
        "fn main() {\n\n    println!(\"hi\");\n}"
    )]
    fn code_block_with_blank_line(#[case] input: &str, #[case] expected_content: &str) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "코드 블록이 분리됨: {:?}", doc);
        let block = &doc.children()[0];
        assert!(block.is_code_block(), "CodeBlock이 아님: {:?}", block);
        assert_eq!(block.content(), expected_content);
    }
}
