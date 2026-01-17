use crate::node::Node;

/// 닫는 # 시퀀스 제거
/// 규칙: 끝에 #들이 있고, 그 앞에 공백이 있으면 제거
fn strip_closing_hashes(s: &str) -> &str {
    // 1. 끝에서 # 제거
    let without_hashes = s.trim_end_matches('#');

    // 2. #이 없었으면 원본 반환
    if without_hashes.len() == s.len() {
        return s;
    }

    // 3. 전체가 #이었으면 (rest가 " ###" 같은 경우)
    //    앞에 공백이 있었다는 것이므로 닫는 시퀀스
    if without_hashes.is_empty() {
        return "";
    }

    // 4. # 앞에 공백/탭이 있는지 확인
    if without_hashes.ends_with(' ') || without_hashes.ends_with('\t') {
        // 공백도 함께 제거
        without_hashes.trim_end()
    } else {
        // 공백 없으면 원본 반환 (# 은 텍스트의 일부)
        s
    }
}

pub fn parse(input: &str) -> Node {
    if input.is_empty() {
        return Node::Document { children: vec![] };
    }

    let children = input.split("\n\n").filter(|s| !s.is_empty()).map(|block| {
        // 앞 들여쓰기 계산 (공백=1, 탭=4, 4칸 이상이면 들여쓰기 코드 블록)
        let indent = block.chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .map(|c| if c == '\t' { 4 } else { 1 })
            .sum::<usize>();
        let block = block.trim();

        // Heading 검사: #로 시작하고, 들여쓰기가 3칸 이하
        if block.starts_with('#') && indent <= 3 {
            // # 개수 세기
            let level = block.chars().take_while(|c| *c == '#').count();

            // 레벨 1~6만 유효, 7개 이상은 Paragraph
            if level >= 1 && level <= 6 {
                let rest = &block[level..];

                // # 뒤에 공백/탭이 있거나 빈 제목이어야 Heading
                if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
                    let content = rest.trim();
                    // 닫는 # 제거
                    let content = strip_closing_hashes(content);
                    return Node::Heading {
                        level: level as u8,
                        children: vec![Node::Text(content.to_string())],
                    };
                }
            }
        }

        // 기본: Paragraph
        Node::Paragraph {
            children: vec![Node::Text(block.to_string())],
        }
    }).collect();

    Node::Document { children }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_string() {
        let doc = parse("");
        assert_eq!(doc.children().len(), 0);
    }

    #[test]
    fn parse_simple_text() {
        let doc = parse("hello");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "hello");
    }

    #[test]
    fn parse_two_paragraphs() {
        let doc = parse("first\n\nsecond");

        assert_eq!(doc.children().len(), 2);
        assert_eq!(doc.children()[0].children()[0].as_text(), "first");
        assert_eq!(doc.children()[1].children()[0].as_text(), "second");
    }

    #[test]
    fn parse_leading_blank_line() {
        let doc = parse("\n\nparagraph");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "paragraph");
    }

    #[test]
    fn parse_trailing_blank_line() {
        let doc = parse("paragraph\n\n");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "paragraph");
    }

    #[test]
    fn parse_multiple_blank_lines() {
        let doc = parse("first\n\n\nsecond");

        assert_eq!(doc.children().len(), 2);
        assert_eq!(doc.children()[0].children()[0].as_text(), "first");
        assert_eq!(doc.children()[1].children()[0].as_text(), "second");
    }

    #[test]
    fn parse_h1_heading() {
        let doc = parse("# heading");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "heading");
    }

    #[test]
    fn parse_heading_requires_space() {
        let doc = parse("#no_space");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "#no_space");
    }

    #[test]
    fn parse_h6_heading() {
        let doc = parse("###### h6 title");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 6);
        assert_eq!(doc.children()[0].children()[0].as_text(), "h6 title");
    }

    #[test]
    fn parse_seven_hashes_is_paragraph() {
        let doc = parse("####### not heading");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "####### not heading");
    }

    // 닫는 # (closing sequence) 테스트
    #[test]
    fn parse_heading_with_closing_hashes() {
        let doc = parse("## foo ##");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 2);
        assert_eq!(doc.children()[0].children()[0].as_text(), "foo");
    }

    #[test]
    fn parse_heading_closing_hashes_count_mismatch() {
        // 닫는 #의 개수는 여는 #과 일치하지 않아도 됨
        let doc = parse("# foo ##########");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "foo");
    }

    #[test]
    fn parse_heading_closing_hash_without_space() {
        // 닫는 # 앞에 공백이 없으면 텍스트의 일부
        let doc = parse("# foo#");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "foo#");
    }

    #[test]
    fn parse_heading_closing_with_text_after() {
        // 닫는 # 뒤에 다른 문자가 있으면 텍스트의 일부
        let doc = parse("### foo ### b");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 3);
        assert_eq!(doc.children()[0].children()[0].as_text(), "foo ### b");
    }

    #[test]
    fn parse_heading_middle_hashes_not_closing() {
        // ## a ## b: 끝이 b이므로 ## b는 닫는 시퀀스가 아님
        let doc = parse("## a ## b");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 2);
        assert_eq!(doc.children()[0].children()[0].as_text(), "a ## b");
    }

    #[test]
    fn parse_heading_closing_with_trailing_spaces() {
        // 닫는 # 뒤에 공백만 있으면 OK
        let doc = parse("### foo ###   ");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 3);
        assert_eq!(doc.children()[0].children()[0].as_text(), "foo");
    }

    #[test]
    fn parse_heading_only_closing_hashes() {
        // ### ### → 빈 h3 (닫는 # 제거 후 빈 내용)
        let doc = parse("### ###");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 3);
        assert_eq!(doc.children()[0].children()[0].as_text(), "");
    }

    #[test]
    fn parse_heading_only_space() {
        // "# " → 빈 h1
        let doc = parse("# ");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "");
    }

    #[test]
    fn parse_empty_heading() {
        // "#" (공백 없음) → 유효한 빈 h1
        let doc = parse("#");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "");
    }

    // 탭 관련 테스트
    #[test]
    fn parse_heading_with_tab_after_hashes() {
        // #\tfoo → 탭도 공백과 동등하게 취급
        let doc = parse("#\tfoo");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "foo");
    }

    #[test]
    fn parse_heading_closing_with_tab() {
        // "# foo\t#" → 탭 뒤 닫는 #
        let doc = parse("# foo\t#");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "foo");
    }

    // 선행 공백 테스트
    #[test]
    fn parse_heading_with_one_leading_space() {
        // " # foo" → 1개 공백 허용
        let doc = parse(" # foo");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "foo");
    }

    #[test]
    fn parse_heading_with_three_leading_spaces() {
        // "   # foo" → 3개 공백까지 허용
        let doc = parse("   # foo");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "foo");
    }

    #[test]
    fn parse_heading_with_four_leading_spaces_is_not_heading() {
        // "    # foo" → 4개 공백은 코드 블록 (현재는 paragraph)
        let doc = parse("    # foo");

        assert_eq!(doc.children().len(), 1);
        // 4개 공백은 heading이 아님 (나중에 코드 블록으로 처리)
        assert_eq!(doc.children()[0].children()[0].as_text(), "# foo");
    }

    #[test]
    fn parse_heading_with_leading_tab_is_not_heading() {
        // "\t# foo" → 탭 = 4칸 공백 → heading이 아님
        let doc = parse("\t# foo");

        assert_eq!(doc.children().len(), 1);
        // 탭은 4칸 공백으로 취급되므로 heading이 아님
        assert_eq!(doc.children()[0].children()[0].as_text(), "# foo");
    }

    #[test]
    fn parse_heading_with_space_and_tab_indent() {
        // "  \t# foo" → 2칸 + 4칸 = 6칸 → heading이 아님
        let doc = parse("  \t# foo");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "# foo");
    }

    #[test]
    fn parse_heading_with_three_spaces_is_heading() {
        // "   # foo" → 3칸 → heading
        let doc = parse("   # foo");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "foo");
    }

    #[test]
    fn parse_heading_with_multiple_spaces_after_hash() {
        // "#    foo" → 여러 공백은 trim됨
        let doc = parse("#    foo");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "foo");
    }

    #[test]
    fn parse_heading_preserves_internal_spaces() {
        // "# foo   bar" → 내부 공백은 유지
        let doc = parse("# foo   bar");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "foo   bar");
    }

    #[test]
    fn parse_heading_with_unicode() {
        // "# 안녕하세요" → 유니코드 제목
        let doc = parse("# 안녕하세요");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "안녕하세요");
    }

    #[test]
    fn parse_heading_with_emoji() {
        // "## 🎉 축하합니다" → 이모지 포함
        let doc = parse("## 🎉 축하합니다");

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].level(), 2);
        assert_eq!(doc.children()[0].children()[0].as_text(), "🎉 축하합니다");
    }
}
