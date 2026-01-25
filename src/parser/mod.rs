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

use crate::node::{
    BlockNode, CodeBlockNode, DocumentNode, HeadingNode, InlineNode, ListItemNode, ListNode,
    ParagraphNode, TextNode,
};
use code_block_fenced::{parse as parse_code_block_fenced, CodeBlockFencedOk};
use code_block_indented::try_start as try_start_code_block_indented;
use context::{
    CodeBlockFencedStart, CodeBlockIndentedStartReason,
    HeadingSetextStartReason, ItemLine, LineResult, ListContinueReason, ListEndReason,
    ListItemStart, ListItemStartReason, NoneContext, ParsingContext,
};
use heading_setext::try_start as try_start_heading_setext;
use helpers::{calculate_indent, trim_blank_lines};

/// 파서 상태: (완성된 노드들, 현재 컨텍스트) - fold 누적용
type ParserState = (Vec<BlockNode>, ParsingContext);

/// 문서 전체 파싱
pub fn parse(input: &str) -> DocumentNode {
    if input.is_empty() {
        return DocumentNode::new(vec![]);
    }

    // fold: 각 줄을 처리하며 상태 전이
    let (children, final_context) = input.lines().fold(
        (Vec::new(), ParsingContext::None(NoneContext)),
        |(children, context), line| process_line(line, context, children),
    );

    // 마지막 컨텍스트 마무리
    let children = finalize_context(final_context, children);

    DocumentNode::new(children)
}

/// 한 줄 처리 후 새 상태 반환
fn process_line(line: &str, context: ParsingContext, nodes: Vec<BlockNode>) -> ParserState {
    let (new_nodes, new_context) = match context {
        ParsingContext::None(ctx) => ctx.parse(line),
        ParsingContext::CodeBlockFenced { start, content } => {
            process_line_in_code_block(line, start, content)
        }
        ParsingContext::Paragraph { pending_lines } => {
            process_line_in_paragraph(line, pending_lines)
        }
        ParsingContext::Blockquote { pending_lines } => {
            process_line_in_blockquote(line, pending_lines)
        }
        ParsingContext::List {
            first_item_start,
            items,
            current_item_lines,
            current_content_indent,
            tight,
            pending_blank_count,
        } => process_line_in_list(line, first_item_start, items, current_item_lines, current_content_indent, tight, pending_blank_count),
        ParsingContext::CodeBlockIndented { pending_lines, pending_blank_count } => {
            process_line_in_code_block_indented(line, pending_lines, pending_blank_count)
        }
    };

    // 새로 완성된 노드들을 누적
    let nodes = extend_nodes(nodes, new_nodes);
    (nodes, new_context)
}

/// 노드 벡터 확장 (불변 스타일)
fn extend_nodes(mut nodes: Vec<BlockNode>, new_nodes: Vec<BlockNode>) -> Vec<BlockNode> {
    nodes.extend(new_nodes);
    nodes
}

/// Code Block 상태에서 줄 처리
/// 반환: (새로 완성된 노드들, 새 컨텍스트)
fn process_line_in_code_block(
    current_line: &str,
    start: CodeBlockFencedStart,
    content: Vec<String>,
) -> LineResult {
    match parse_code_block_fenced(current_line, Some(&start)).unwrap() {
        CodeBlockFencedOk::End => {
            let node = code_block_fenced::finalize(start, content);
            (vec![node], ParsingContext::None(NoneContext))
        }
        CodeBlockFencedOk::Content(line) => {
            let content = push_string(content, line);
            let context = ParsingContext::CodeBlockFenced { start, content };
            (vec![], context)
        }
        CodeBlockFencedOk::Start(_) => unreachable!("parse with Some context should return End or Content"),
    }
}

/// Paragraph 상태에서 줄 처리
/// 반환: (새로 완성된 노드들, 새 컨텍스트)
fn process_line_in_paragraph(current_line: &str, pending_lines: Vec<String>) -> LineResult {
    // 빈 줄이면 Paragraph 종료
    if current_line.trim().is_empty() {
        let text = pending_lines.join("\n");
        return (vec![paragraph::parse(&text)], ParsingContext::None(NoneContext));
    }

    // Fenced Code Block 시작이면 Paragraph 종료 후 Code Block 시작
    if let Ok(CodeBlockFencedOk::Start(start)) = parse_code_block_fenced(current_line, None) {
        let text = pending_lines.join("\n");
        let context = ParsingContext::CodeBlockFenced {
            start,
            content: Vec::new(),
        };
        return (vec![paragraph::parse(&text)], context);
    }

    let trimmed = current_line.trim();
    let indent = calculate_indent(current_line);

    // Setext Heading 밑줄이면 Paragraph를 Heading으로 변환
    // 중요: Thematic Break보다 먼저 확인해야 함 (---가 Setext 밑줄로 해석됨)
    if let Ok(HeadingSetextStartReason::Started(start)) = try_start_heading_setext(trimmed, indent) {
        let text = pending_lines.join("\n");
        let node = BlockNode::Heading(HeadingNode::new(
            start.level.to_level(),
            vec![InlineNode::Text(TextNode::new(&text))],
        ));
        return (vec![node], ParsingContext::None(NoneContext));
    }

    // Thematic Break이면 Paragraph 종료
    if let Ok(node) = thematic_break::parse(current_line) {
        let text = pending_lines.join("\n");
        return (vec![paragraph::parse(&text), node], ParsingContext::None(NoneContext));
    }

    // ATX Heading이면 Paragraph 종료
    if let Ok(node) = heading::parse(current_line) {
        let text = pending_lines.join("\n");
        return (vec![paragraph::parse(&text), node], ParsingContext::None(NoneContext));
    }

    // Blockquote 시작이면 Paragraph 종료 후 Blockquote 시작
    if let Ok(content) = blockquote::parse(current_line) {
        let text = pending_lines.join("\n");
        let context = ParsingContext::Blockquote {
            pending_lines: vec![content],
        };
        return (vec![paragraph::parse(&text)], context);
    }

    // List 시작이면 Paragraph 종료 후 List 시작
    // CommonMark: List는 Paragraph를 인터럽트 가능 (단, 빈 아이템 제외)
    if let Ok(ListItemStartReason::Started(start)) = list_item::try_start(current_line) {
        // 빈 아이템은 Paragraph 인터럽트 불가 (CommonMark 명세)
        if !start.content.is_empty() {
            let text = pending_lines.join("\n");
            let content_indent = start.content_indent;
            let context = ParsingContext::List {
                first_item_start: start.clone(),
                items: Vec::new(),
                current_item_lines: vec![ItemLine::text(start.content)],
                current_content_indent: content_indent,
                tight: true,
                pending_blank_count: 0,
            };
            return (vec![paragraph::parse(&text)], context);
        }
    }

    // 줄 추가
    let pending_lines = push_string(pending_lines, current_line.trim().to_string());
    (vec![], ParsingContext::Paragraph { pending_lines })
}

/// List 상태에서 줄 처리
/// 반환: (새로 완성된 노드들, 새 컨텍스트)
fn process_line_in_list(
    current_line: &str,
    first_item_start: ListItemStart,
    items: Vec<Vec<ItemLine>>,
    current_item_lines: Vec<ItemLine>,
    current_content_indent: usize,
    tight: bool,
    pending_blank_count: usize,
) -> LineResult {
    // Example 301: 새 아이템 판단은 current_content_indent 기준
    // Example 303: continuation 판단은 first_item의 content_indent 기준
    match list_item::try_end(
        current_line,
        &first_item_start.marker,
        first_item_start.content_indent,
        current_content_indent,
    ) {
        // 종료: List 노드 생성 + 현재 줄 재처리
        Ok(ListEndReason::Reprocess) => {
            let list_node = build_list_node(&first_item_start, items, current_item_lines, tight);
            let (more_nodes, new_context) = NoneContext.parse(current_line);
            let mut nodes = vec![list_node];
            nodes.extend(more_nodes);
            (nodes, new_context)
        }
        // 계속: 빈 줄
        Err(ListContinueReason::Blank) => {
            let context = ParsingContext::List {
                first_item_start,
                items,
                current_item_lines,
                current_content_indent,
                tight,
                pending_blank_count: pending_blank_count + 1,
            };
            (vec![], context)
        }
        // 계속: 새 아이템
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
            (vec![], context)
        }
        // 계속: continuation line
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
            (vec![], context)
        }
    }
}

/// List 노드 생성 (완성된 아이템들로부터)
fn build_list_node(
    first_item_start: &ListItemStart,
    items: Vec<Vec<ItemLine>>,
    current_item_lines: Vec<ItemLine>,
    tight: bool,
) -> BlockNode {
    let (list_type, start) = first_item_start.marker.to_list_type();
    let all_items = push_item(items, current_item_lines);

    // 각 아이템을 파싱하여 ListItem 노드 생성
    let list_children: Vec<ListItemNode> = all_items
        .iter()
        .map(|item_lines| {
            let parsed_blocks = parse_item_lines(item_lines);
            ListItemNode::new(parsed_blocks)
        })
        .collect();

    BlockNode::List(ListNode::new(list_type, start, tight, list_children))
}

/// 리스트 아이템 내용 파싱
/// text_only 플래그를 고려하여 처리
fn parse_item_lines(lines: &[ItemLine]) -> Vec<BlockNode> {
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
        doc.children
    }
}

/// text_only가 있는 아이템 내용 파싱
fn parse_item_lines_with_text_only(lines: &[ItemLine]) -> Vec<BlockNode> {
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

    let mut result: Vec<BlockNode> = vec![];

    for (chunk, has_text_only) in chunks {
        let content: String = chunk.iter().map(|l| l.content.as_str()).collect::<Vec<_>>().join("\n");

        if has_text_only {
            // text_only가 있는 청크는 무조건 paragraph로 처리
            result.push(BlockNode::Paragraph(ParagraphNode::new(vec![
                InlineNode::Text(TextNode::new(&content)),
            ])));
        } else {
            // 일반 청크는 전체 파서로 파싱
            let doc = parse(&content);
            result.extend(doc.children);
        }
    }

    result
}

/// Indented Code Block 상태에서 줄 처리
/// 반환: (새로 완성된 노드들, 새 컨텍스트)
fn process_line_in_code_block_indented(
    current_line: &str,
    pending_lines: Vec<String>,
    pending_blank_count: usize,
) -> LineResult {
    use context::CodeBlockIndentedNotStartReason;

    match try_start_code_block_indented(current_line) {
        // 4칸 이상 들여쓰기 → 코드 줄 추가
        Ok(CodeBlockIndentedStartReason::Started(start)) => {
            let mut pending_lines = pending_lines;
            for _ in 0..pending_blank_count {
                pending_lines = push_string(pending_lines, String::new());
            }
            let pending_lines = push_string(pending_lines, start.content);
            let context = ParsingContext::CodeBlockIndented {
                pending_lines,
                pending_blank_count: 0,
            };
            (vec![], context)
        }
        // 4칸 미만 빈 줄 → 대기 (코드 블록 종료 여부는 다음 줄에서 결정)
        Err(CodeBlockIndentedNotStartReason::Empty) => {
            let context = ParsingContext::CodeBlockIndented {
                pending_lines,
                pending_blank_count: pending_blank_count + 1,
            };
            (vec![], context)
        }
        // 4칸 미만 비빈 줄 → 코드 블록 종료
        Err(CodeBlockIndentedNotStartReason::InsufficientIndent) => {
            let content = trim_blank_lines(pending_lines);
            let code_node = BlockNode::CodeBlock(CodeBlockNode::new(None, content));
            // 현재 줄을 다시 처리
            let (more_nodes, new_context) = NoneContext.parse(current_line);
            let mut nodes = vec![code_node];
            nodes.extend(more_nodes);
            (nodes, new_context)
        }
    }
}

/// 아이템 리스트에 아이템 추가
fn push_item(mut items: Vec<Vec<ItemLine>>, item: Vec<ItemLine>) -> Vec<Vec<ItemLine>> {
    items.push(item);
    items
}

/// Blockquote 상태에서 줄 처리
/// 반환: (새로 완성된 노드들, 새 컨텍스트)
fn process_line_in_blockquote(current_line: &str, pending_lines: Vec<String>) -> LineResult {
    let trimmed = current_line.trim();

    // 빈 줄이면 Blockquote 종료
    if trimmed.is_empty() {
        let node = blockquote::finalize(pending_lines, parse_block_simple);
        return (vec![node], ParsingContext::None(NoneContext));
    }

    // Fenced Code Block 시작이면 Blockquote 종료
    if let Ok(CodeBlockFencedOk::Start(start)) = parse_code_block_fenced(current_line, None) {
        let node = blockquote::finalize(pending_lines, parse_block_simple);
        let context = ParsingContext::CodeBlockFenced {
            start,
            content: Vec::new(),
        };
        return (vec![node], context);
    }

    // Thematic Break이면 Blockquote 종료
    if let Ok(node) = thematic_break::parse(current_line) {
        let bq_node = blockquote::finalize(pending_lines, parse_block_simple);
        return (vec![bq_node, node], ParsingContext::None(NoneContext));
    }

    // ATX Heading이면 Blockquote 종료
    if let Ok(node) = heading::parse(current_line) {
        let bq_node = blockquote::finalize(pending_lines, parse_block_simple);
        return (vec![bq_node, node], ParsingContext::None(NoneContext));
    }

    // > 로 시작하면 마커 제거 후 저장, 아니면 lazy continuation
    let content = match blockquote::parse(current_line) {
        Ok(stripped) => stripped,
        Err(_) => trimmed.to_string(),
    };
    let pending_lines = push_string(pending_lines, content);
    (vec![], ParsingContext::Blockquote { pending_lines })
}

/// 마지막 컨텍스트 마무리
fn finalize_context(context: ParsingContext, nodes: Vec<BlockNode>) -> Vec<BlockNode> {
    match context {
        ParsingContext::None(NoneContext) => nodes,
        ParsingContext::CodeBlockFenced { start, content } => {
            let node = code_block_fenced::finalize(start, content);
            push_node(nodes, node)
        }
        ParsingContext::Paragraph { pending_lines } => {
            let text = pending_lines.join("\n");
            push_node(nodes, paragraph::parse(&text))
        }
        ParsingContext::Blockquote { pending_lines } => {
            let node = blockquote::finalize(pending_lines, parse_block_simple);
            push_node(nodes, node)
        }
        ParsingContext::List {
            first_item_start,
            items,
            current_item_lines,
            current_content_indent: _,
            tight,
            pending_blank_count: _,
        } => {
            let list_node = build_list_node(&first_item_start, items, current_item_lines, tight);
            push_node(nodes, list_node)
        }
        ParsingContext::CodeBlockIndented { pending_lines, pending_blank_count: _ } => {
            let content = trim_blank_lines(pending_lines);
            let node = BlockNode::CodeBlock(CodeBlockNode::new(None, content));
            push_node(nodes, node)
        }
    }
}

/// 벡터에 요소 추가 후 반환 (불변 스타일)
fn push_node(mut vec: Vec<BlockNode>, node: BlockNode) -> Vec<BlockNode> {
    vec.push(node);
    vec
}

/// 문자열 벡터에 요소 추가 후 반환
fn push_string(mut vec: Vec<String>, s: String) -> Vec<String> {
    vec.push(s);
    vec
}

/// 단일 블록 파싱 (blockquote 내부 등에서 사용)
fn parse_block_simple(block: &str) -> BlockNode {
    if let Some(node) = code_block_fenced::parse_text(block) {
        return node;
    }

    // 중첩 blockquote 처리를 위해 blockquote 파싱 시도
    if let Some(node) = blockquote::parse_text(block, parse_block_simple) {
        return node;
    }

    if let Ok(node) = thematic_break::parse(block) {
        return node;
    }

    if let Ok(node) = heading::parse(block) {
        return node;
    }

    paragraph::parse(block.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_string() {
        let doc = parse("");
        assert_eq!(doc.children.len(), 0);
    }
}
