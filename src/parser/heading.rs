//! https://spec.commonmark.org/0.31.2/#atx-headings

use super::helpers::{calculate_indent, count_leading_char};

#[derive(Debug, Clone, PartialEq)]
pub struct HeadingOkReason {
    pub level: u8,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeadingErrReason {
    /// 4칸 이상 들여쓰기 (코드 블록으로 해석됨)
    CodeBlockIndented,
    /// #로 시작하지 않음
    NotHashStart,
    /// # 개수 초과 (7개 이상)
    TooManyHashes,
    /// # 뒤에 공백 없음
    NoSpaceAfterHashes,
}

pub fn parse(line: &str) -> Result<HeadingOkReason, HeadingErrReason> {
    let indent = calculate_indent(line);
    let trimmed = line.trim();

    // 들여쓰기 3칸 초과면 코드 블록
    if indent > 3 {
        return Err(HeadingErrReason::CodeBlockIndented);
    }

    // #로 시작하지 않으면 Heading 아님
    if !trimmed.starts_with('#') {
        return Err(HeadingErrReason::NotHashStart);
    }

    // # 개수 세기
    let level = count_leading_char(trimmed, '#');

    // 레벨 1~6만 유효
    if level > 6 {
        return Err(HeadingErrReason::TooManyHashes);
    }

    let rest = &trimmed[level..];

    // # 뒤에 공백/탭이 있거나 빈 제목이어야 Heading
    if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
        let content = rest.trim();
        let content = strip_closing_hashes(content);
        Ok(HeadingOkReason {
            level: level as u8,
            content: content.to_string(),
        })
    } else {
        Err(HeadingErrReason::NoSpaceAfterHashes)
    }
}

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

#[cfg(test)]
mod tests {
    use crate::node::Node;
    use crate::parser::parse;
    use rstest::rstest;

    #[rstest]
    // Example 62: 모든 레벨 h1-h6
    #[case("# foo", vec![Node::heading(1, vec![Node::text("foo")])])]
    #[case("## foo", vec![Node::heading(2, vec![Node::text("foo")])])]
    #[case("### foo", vec![Node::heading(3, vec![Node::text("foo")])])]
    #[case("#### foo", vec![Node::heading(4, vec![Node::text("foo")])])]
    #[case("##### foo", vec![Node::heading(5, vec![Node::text("foo")])])]
    #[case("###### foo", vec![Node::heading(6, vec![Node::text("foo")])])]
    // Example 63: 7개 이상 # → Paragraph
    #[case("####### foo", vec![Node::para(vec![Node::text("####### foo")])])]
    // Example 64: # 뒤 공백 없음 → Paragraph
    #[case("#5 bolt", vec![Node::para(vec![Node::text("#5 bolt")])])]
    #[case("#hashtag", vec![Node::para(vec![Node::text("#hashtag")])])]
    // Example 67: # 뒤 여러 공백
    #[case("#                  foo", vec![Node::heading(1, vec![Node::text("foo")])])]
    // Example 68: 1-3칸 들여쓰기 허용
    #[case(" ### foo", vec![Node::heading(3, vec![Node::text("foo")])])]
    #[case("  ## foo", vec![Node::heading(2, vec![Node::text("foo")])])]
    #[case("   # foo", vec![Node::heading(1, vec![Node::text("foo")])])]
    // Example 69: 4칸 들여쓰기는 코드 블록
    #[case("    # foo", vec![Node::code_block(None, "# foo")])]
    // Example 70: Paragraph 내 4칸 들여쓰기는 continuation
    #[case("foo\n    # bar", vec![Node::para(vec![Node::text("foo\n# bar")])])]
    // Example 71: 닫는 # 시퀀스
    #[case("## foo ##", vec![Node::heading(2, vec![Node::text("foo")])])]
    #[case("  ###   bar    ###", vec![Node::heading(3, vec![Node::text("bar")])])]
    // Example 72: 많은 닫는 #
    #[case("# foo ##################################", vec![Node::heading(1, vec![Node::text("foo")])])]
    #[case("##### foo ##", vec![Node::heading(5, vec![Node::text("foo")])])]
    // Example 73: 닫는 # 뒤 공백
    #[case("### foo ###     ", vec![Node::heading(3, vec![Node::text("foo")])])]
    // Example 74: 닫는 # 뒤 텍스트
    #[case("### foo ### b", vec![Node::heading(3, vec![Node::text("foo ### b")])])]
    // Example 75: # 앞 공백 없음
    #[case("# foo#", vec![Node::heading(1, vec![Node::text("foo#")])])]
    // Example 77: Heading이 thematic break 인터럽트
    #[case("****\n## foo\n****", vec![Node::ThematicBreak, Node::heading(2, vec![Node::text("foo")]), Node::ThematicBreak])]
    // Example 78: Heading이 paragraph 인터럽트
    #[case("Foo bar\n# baz\nBar foo", vec![Node::para(vec![Node::text("Foo bar")]), Node::heading(1, vec![Node::text("baz")]), Node::para(vec![Node::text("Bar foo")])])]
    // Example 79: 빈 heading
    #[case("##", vec![Node::heading(2, vec![Node::text("")])])]
    #[case("#", vec![Node::heading(1, vec![Node::text("")])])]
    #[case("### ###", vec![Node::heading(3, vec![Node::text("")])])]
    // 추가 케이스
    #[case("# heading", vec![Node::heading(1, vec![Node::text("heading")])])]
    #[case("###### h6 title", vec![Node::heading(6, vec![Node::text("h6 title")])])]
    #[case("# ", vec![Node::heading(1, vec![Node::text("")])])]
    #[case("## a ## b", vec![Node::heading(2, vec![Node::text("a ## b")])])]
    #[case("#\tfoo", vec![Node::heading(1, vec![Node::text("foo")])])]
    #[case("# foo\t#", vec![Node::heading(1, vec![Node::text("foo")])])]
    #[case(" # foo", vec![Node::heading(1, vec![Node::text("foo")])])]
    #[case("#    foo", vec![Node::heading(1, vec![Node::text("foo")])])]
    #[case("# foo   bar", vec![Node::heading(1, vec![Node::text("foo   bar")])])]
    #[case("# 안녕하세요", vec![Node::heading(1, vec![Node::text("안녕하세요")])])]
    #[case("## 🎉 축하합니다", vec![Node::heading(2, vec![Node::text("🎉 축하합니다")])])]
    #[case("#no_space", vec![Node::para(vec![Node::text("#no_space")])])]
    fn test_heading(#[case] input: &str, #[case] expected: Vec<Node>) {
        let doc = parse(input);
        assert_eq!(doc.children(), &expected);
    }
}
