//! NoneContext: 새 블록 시작 대기 상태

use super::{
    CodeBlockFencedStartReason, CodeBlockIndentedStartReason, ItemLine, LineResult,
    ListItemStartReason, ParsingContext,
};
use crate::parser::code_block_fenced::try_start as try_start_code_block_fenced;
use crate::parser::code_block_indented::try_start as try_start_code_block_indented;
use crate::parser::helpers::calculate_indent;
use crate::parser::{heading, list_item, thematic_break};

#[derive(Debug, Clone, Default)]
pub struct NoneContext;

impl NoneContext {
    pub fn parse(self, current_line: &str) -> LineResult {
        let indent = calculate_indent(current_line);
        let trimmed = current_line.trim();

        // 빈 줄은 무시
        if trimmed.is_empty() {
            return (vec![], ParsingContext::None(NoneContext));
        }

        // 한 줄 블록들 (Thematic Break, ATX Heading)
        if let Some(node) = thematic_break::parse(trimmed, indent) {
            return (vec![node], ParsingContext::None(NoneContext));
        }

        if let Some(node) = heading::parse(trimmed, indent) {
            return (vec![node], ParsingContext::None(NoneContext));
        }

        // Fenced Code Block 시작 감지
        if let Ok(CodeBlockFencedStartReason::Started(start)) =
            try_start_code_block_fenced(current_line)
        {
            let context = ParsingContext::CodeBlockFenced {
                start,
                content: Vec::new(),
            };
            return (vec![], context);
        }

        // Blockquote 시작 감지 (> 로 시작하고 들여쓰기 3칸 이하)
        if trimmed.starts_with('>') && indent <= 3 {
            let context = ParsingContext::Blockquote {
                pending_lines: vec![trimmed.to_string()],
            };
            return (vec![], context);
        }

        // List 시작 감지
        if let Ok(ListItemStartReason::Started(start)) = list_item::try_start(current_line) {
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
            return (vec![], context);
        }

        // Indented Code Block 시작 감지 (List 후에 체크 - 명세상 List가 우선)
        if let Ok(CodeBlockIndentedStartReason::Started(start)) =
            try_start_code_block_indented(current_line)
        {
            let context = ParsingContext::CodeBlockIndented {
                pending_lines: vec![start.content],
                pending_blank_count: 0,
            };
            return (vec![], context);
        }

        // 나머지는 Paragraph 시작
        let context = ParsingContext::Paragraph {
            pending_lines: vec![current_line.trim().to_string()],
        };
        (vec![], context)
    }
}
