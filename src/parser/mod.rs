//! CommonMark 파서
//!
//! 라인 단위로 스캔하며 블록 레벨 요소를 파싱합니다.
//! fold 패턴을 사용하여 불변 상태 전이를 구현합니다.

mod blockquote;
mod code_block_fenced;
mod code_block_indented;
mod context;
mod heading;
mod heading_setext;
mod helpers;
mod list;
mod list_item;
mod paragraph;
mod thematic_break;

use crate::node::Node;
use code_block_fenced::{
    try_end as try_end_code_block_fenced, try_start as try_start_code_block_fenced,
};
use code_block_indented::try_start as try_start_code_block_indented;
use context::{
    CodeBlockFencedStart, CodeBlockFencedStartReason, CodeBlockIndentedStartReason,
    HeadingSetextStartReason, ItemLine, ListContinueReason, ListEndReason, ListItemStart,
    ListItemStartReason, ParsingContext,
};
use heading_setext::try_start as try_start_heading_setext;
use helpers::{calculate_indent, remove_indent, trim_blank_lines};

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
        ParsingContext::CodeBlockFenced { start, content } => {
            process_line_in_code_block(line, start, content, children)
        }
        ParsingContext::Paragraph { lines } => process_line_in_paragraph(line, lines, children),
        ParsingContext::Blockquote { lines } => process_line_in_blockquote(line, lines, children),
        ParsingContext::List {
            first_item_start,
            items,
            current_item_lines,
            current_content_indent,
            tight,
            pending_blank_count,
        } => process_line_in_list(line, first_item_start, items, current_item_lines, current_content_indent, tight, pending_blank_count, children),
        ParsingContext::CodeBlockIndented { lines, pending_blank_count } => {
            process_line_in_code_block_indented(line, lines, pending_blank_count, children)
        }
    }
}

/// None 상태에서 줄 처리: 새 블록 시작 감지
fn process_line_in_none(line: &str, children: Vec<Node>) -> ParserState {
    // 빈 줄은 무시
    if line.trim().is_empty() {
        return (children, ParsingContext::None);
    }

    // Fenced Code Block 시작 감지
    if let Ok(CodeBlockFencedStartReason::Started(start)) = try_start_code_block_fenced(line) {
        let context = ParsingContext::CodeBlockFenced {
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
    if let Ok(ListItemStartReason::Started(start)) = list_item::try_start(line) {
        let content = start.content.clone();
        let content_indent = start.content_indent;
        let context = ParsingContext::List {
            first_item_start: start,
            items: Vec::new(),
            current_item_lines: vec![ItemLine::text(content)],
            current_content_indent: content_indent,
            tight: true,
            pending_blank_count: 0,
        };
        return (children, context);
    }

    // Indented Code Block 시작 감지 (List 후에 체크 - 명세상 List가 우선)
    if let Ok(CodeBlockIndentedStartReason::Started(start)) = try_start_code_block_indented(line) {
        let context = ParsingContext::CodeBlockIndented {
            lines: vec![start.content],
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
    start: CodeBlockFencedStart,
    content: Vec<String>,
    children: Vec<Node>,
) -> ParserState {
    // 닫는 펜스인지 확인
    if try_end_code_block_fenced(line, start.fence_char, start.fence_len).is_ok() {
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

    let context = ParsingContext::CodeBlockFenced { start, content };
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
    if let Ok(CodeBlockFencedStartReason::Started(start)) = try_start_code_block_fenced(line) {
        let text = lines.join("\n");
        let children = push_node(children, paragraph::parse(&text));
        let context = ParsingContext::CodeBlockFenced {
            start,
            content: Vec::new(),
        };
        return (children, context);
    }

    let trimmed = line.trim();
    let indent = calculate_indent(line);

    // Setext Heading 밑줄이면 Paragraph를 Heading으로 변환
    // 중요: Thematic Break보다 먼저 확인해야 함 (---가 Setext 밑줄로 해석됨)
    if let Ok(HeadingSetextStartReason::Started(start)) = try_start_heading_setext(trimmed, indent) {
        let text = lines.join("\n");
        let node = Node::Heading {
            level: start.level.to_level(),
            children: vec![Node::Text(text)],
        };
        let children = push_node(children, node);
        return (children, ParsingContext::None);
    }

    // Thematic Break이면 Paragraph 종료
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

    // List 시작이면 Paragraph 종료 후 List 시작
    // CommonMark: List는 Paragraph를 인터럽트 가능 (단, 빈 아이템 제외)
    if let Ok(ListItemStartReason::Started(start)) = list_item::try_start(line) {
        // 빈 아이템은 Paragraph 인터럽트 불가 (CommonMark 명세)
        if !start.content.is_empty() {
            let text = lines.join("\n");
            let children = push_node(children, paragraph::parse(&text));
            let content_indent = start.content_indent;
            let context = ParsingContext::List {
                first_item_start: start.clone(),
                items: Vec::new(),
                current_item_lines: vec![ItemLine::text(start.content)],
                current_content_indent: content_indent,
                tight: true,
                pending_blank_count: 0,
            };
            return (children, context);
        }
    }

    // 줄 추가
    let lines = push_string(lines, line.trim().to_string());
    (children, ParsingContext::Paragraph { lines })
}

/// List 상태에서 줄 처리
fn process_line_in_list(
    line: &str,
    first_item_start: ListItemStart,
    items: Vec<Vec<ItemLine>>,
    current_item_lines: Vec<ItemLine>,
    current_content_indent: usize,
    tight: bool,
    pending_blank_count: usize,
    children: Vec<Node>,
) -> ParserState {
    // Example 301: 새 아이템 판단은 current_content_indent 기준
    // Example 303: continuation 판단은 first_item의 content_indent 기준
    match list_item::try_end(
        line,
        &first_item_start.marker,
        first_item_start.content_indent,
        current_content_indent,
    ) {
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
                current_content_indent,
                tight,
                pending_blank_count: pending_blank_count + 1,
            };
            (children, context)
        }
        Err(ListContinueReason::NewItem(new_start)) => {
            let items = push_item(items, current_item_lines);
            // 빈 줄이 있었으면 loose list
            let tight = tight && pending_blank_count == 0;
            // 새 아이템의 content_indent로 업데이트
            let new_content_indent = new_start.content_indent;
            let context = ParsingContext::List {
                first_item_start,
                items,
                current_item_lines: vec![ItemLine::text(new_start.content)],
                current_content_indent: new_content_indent,
                tight,
                pending_blank_count: 0,
            };
            (children, context)
        }
        Err(ListContinueReason::ContinuationLine(item_line)) => {
            // 대기 중인 빈 줄을 내용에 추가
            let mut lines = current_item_lines;
            for _ in 0..pending_blank_count {
                lines.push(ItemLine::blank());
            }
            lines.push(item_line);
            // 아이템 내에 빈 줄이 있었으면 loose list
            let tight = tight && pending_blank_count == 0;
            let context = ParsingContext::List {
                first_item_start,
                items,
                current_item_lines: lines,
                current_content_indent,
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
    items: Vec<Vec<ItemLine>>,
    current_item_lines: Vec<ItemLine>,
    tight: bool,
    children: Vec<Node>,
) -> Vec<Node> {
    let (list_type, start) = first_item_start.marker.to_list_type();
    let all_items = push_item(items, current_item_lines);

    // 각 아이템을 파싱하여 ListItem 노드 생성
    let list_children: Vec<Node> = all_items
        .iter()
        .map(|item_lines| {
            let parsed_blocks = parse_item_lines(item_lines);
            Node::ListItem {
                children: parsed_blocks,
            }
        })
        .collect();

    let list_node = Node::List {
        list_type,
        start,
        tight,
        children: list_children,
    };

    push_node(children, list_node)
}

/// 리스트 아이템 내용 파싱
/// text_only 플래그를 고려하여 처리
fn parse_item_lines(lines: &[ItemLine]) -> Vec<Node> {
    // text_only가 있는지 확인
    let has_any_text_only = lines.iter().any(|l| l.text_only);

    if has_any_text_only {
        // text_only가 있는 경우: 청크 단위로 처리
        // 빈 줄로 분리하되, 빈 줄 후 들여쓰기된 내용은 이전 청크에 포함
        parse_item_lines_with_text_only(lines)
    } else {
        // text_only가 없는 경우: 전체를 한 번에 재파싱
        // 빈 줄이 있어도 리스트 continuation으로 처리됨
        let content: String = lines.iter().map(|l| l.content.as_str()).collect::<Vec<_>>().join("\n");
        let doc = parse(&content);
        match doc {
            Node::Document { children } => children,
            _ => vec![doc],
        }
    }
}

/// text_only가 있는 아이템 내용 파싱
fn parse_item_lines_with_text_only(lines: &[ItemLine]) -> Vec<Node> {
    // 빈 줄을 기준으로 청크로 분리
    let mut chunks: Vec<(Vec<&ItemLine>, bool)> = vec![]; // (lines, has_text_only)
    let mut current_chunk: Vec<&ItemLine> = vec![];
    let mut current_has_text_only = false;

    for line in lines {
        if line.content.trim().is_empty() && !line.text_only {
            if !current_chunk.is_empty() {
                chunks.push((current_chunk, current_has_text_only));
                current_chunk = vec![];
                current_has_text_only = false;
            }
        } else {
            if line.text_only {
                current_has_text_only = true;
            }
            current_chunk.push(line);
        }
    }
    if !current_chunk.is_empty() {
        chunks.push((current_chunk, current_has_text_only));
    }

    let mut result: Vec<Node> = vec![];

    for (chunk, has_text_only) in chunks {
        let content: String = chunk.iter().map(|l| l.content.as_str()).collect::<Vec<_>>().join("\n");

        if has_text_only {
            // text_only가 있는 청크는 무조건 paragraph로 처리
            result.push(Node::Paragraph {
                children: vec![Node::Text(content)],
            });
        } else {
            // 일반 청크는 전체 파서로 파싱
            let doc = parse(&content);
            match doc {
                Node::Document { children } => result.extend(children),
                _ => result.push(doc),
            }
        }
    }

    result
}

/// Indented Code Block 상태에서 줄 처리
fn process_line_in_code_block_indented(
    line: &str,
    lines: Vec<String>,
    pending_blank_count: usize,
    children: Vec<Node>,
) -> ParserState {
    use context::CodeBlockIndentedNotStartReason;

    match try_start_code_block_indented(line) {
        // 4칸 이상 들여쓰기 → 코드 줄 추가
        Ok(CodeBlockIndentedStartReason::Started(start)) => {
            let mut lines = lines;
            for _ in 0..pending_blank_count {
                lines = push_string(lines, String::new());
            }
            let lines = push_string(lines, start.content);
            let context = ParsingContext::CodeBlockIndented {
                lines,
                pending_blank_count: 0,
            };
            (children, context)
        }
        // 4칸 미만 빈 줄 → 대기 (코드 블록 종료 여부는 다음 줄에서 결정)
        Err(CodeBlockIndentedNotStartReason::Empty) => {
            let context = ParsingContext::CodeBlockIndented {
                lines,
                pending_blank_count: pending_blank_count + 1,
            };
            (children, context)
        }
        // 4칸 미만 비빈 줄 → 코드 블록 종료
        Err(CodeBlockIndentedNotStartReason::InsufficientIndent) => {
            let content = trim_blank_lines(lines);
            let node = Node::CodeBlock { info: None, content };
            let children = push_node(children, node);
            // 현재 줄을 다시 처리
            process_line_in_none(line, children)
        }
    }
}

/// 아이템 리스트에 아이템 추가
fn push_item(mut items: Vec<Vec<ItemLine>>, item: Vec<ItemLine>) -> Vec<Vec<ItemLine>> {
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
    if let Ok(CodeBlockFencedStartReason::Started(start)) = try_start_code_block_fenced(line) {
        let text = lines.join("\n");
        let children = if let Some(node) = blockquote::parse(&text, 0, parse_block_simple) {
            push_node(children, node)
        } else {
            children
        };
        let context = ParsingContext::CodeBlockFenced {
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
        ParsingContext::CodeBlockFenced { start, content } => {
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
            current_content_indent: _,
            tight,
            pending_blank_count: _,
        } => finalize_list(&first_item_start, items, current_item_lines, tight, children),
        ParsingContext::CodeBlockIndented { lines, pending_blank_count: _ } => {
            let content = trim_blank_lines(lines);
            let node = Node::CodeBlock { info: None, content };
            push_node(children, node)
        }
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

    if let Some(node) = code_block_fenced::parse(block, indent) {
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

    #[test]
    fn parse_empty_string() {
        let doc = parse("");
        assert_eq!(doc.children().len(), 0);
    }
}
