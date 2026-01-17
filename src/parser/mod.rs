//! CommonMark 파서
//!
//! 블록 레벨 요소를 파싱하여 AST를 생성합니다.

mod blockquote;
mod fenced_code_block;
mod heading;
mod paragraph;
mod thematic_break;

use crate::node::Node;

/// 문서 전체 파싱
pub fn parse(input: &str) -> Node {
    if input.is_empty() {
        return Node::Document { children: vec![] };
    }

    let children = input
        .split("\n\n")
        .filter(|s| !s.is_empty())
        .map(parse_block)
        .collect();

    Node::Document { children }
}

/// 단일 블록 파싱
fn parse_block(block: &str) -> Node {
    let indent = calculate_indent(block);
    let trimmed = block.trim();

    // 순서대로 파싱 시도, 첫 번째 성공 반환
    // 주의: fenced_code_block은 trimmed 대신 block을 사용 (들여쓰기 보존)
    fenced_code_block::parse(block, indent)
        .or_else(|| thematic_break::parse(trimmed, indent))
        .or_else(|| blockquote::parse(trimmed, indent, parse_block))
        .or_else(|| heading::parse(trimmed, indent))
        .unwrap_or_else(|| paragraph::parse(trimmed))
}

/// 들여쓰기 계산 (공백=1, 탭=4)
fn calculate_indent(block: &str) -> usize {
    block
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
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
