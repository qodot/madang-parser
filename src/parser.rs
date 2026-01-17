use crate::node::Node;

/// Thematic Break 검사
/// 규칙: *, -, _ 중 하나가 3개 이상, 공백/탭만 사이에 허용
fn is_thematic_break(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }

    // 첫 문자가 마커 문자인지 확인
    let first = trimmed.chars().next().unwrap();
    if first != '*' && first != '-' && first != '_' {
        return false;
    }

    // 모든 문자가 같은 마커이거나 공백/탭인지 확인
    let mut marker_count = 0;
    for c in trimmed.chars() {
        if c == first {
            marker_count += 1;
        } else if c != ' ' && c != '\t' {
            return false;
        }
    }

    // 마커가 3개 이상이어야 함
    marker_count >= 3
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

/// 단일 블록 파싱
fn parse_block(block: &str) -> Node {
    // 앞 들여쓰기 계산 (공백=1, 탭=4)
    let indent = block.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum::<usize>();
    let trimmed = block.trim();

    // Thematic Break 검사: 들여쓰기 3칸 이하
    if indent <= 3 && is_thematic_break(trimmed) {
        return Node::ThematicBreak;
    }

    // Blockquote 검사: >로 시작하고, 들여쓰기가 3칸 이하
    if trimmed.starts_with('>') && indent <= 3 {
        // > 다음 내용 추출 (공백 하나 건너뛰기)
        let rest = &trimmed[1..];
        let content = if rest.starts_with(' ') || rest.starts_with('\t') {
            &rest[1..]
        } else {
            rest
        };
        // 재귀적으로 내용 파싱
        let inner = parse_block(content);
        return Node::Blockquote {
            children: vec![inner],
        };
    }

    // Heading 검사: #로 시작하고, 들여쓰기가 3칸 이하
    if trimmed.starts_with('#') && indent <= 3 {
        // # 개수 세기
        let level = trimmed.chars().take_while(|c| *c == '#').count();

        // 레벨 1~6만 유효, 7개 이상은 Paragraph
        if level >= 1 && level <= 6 {
            let rest = &trimmed[level..];

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
        children: vec![Node::Text(trimmed.to_string())],
    }
}

pub fn parse(input: &str) -> Node {
    if input.is_empty() {
        return Node::Document { children: vec![] };
    }

    let children = input.split("\n\n")
        .filter(|s| !s.is_empty())
        .map(parse_block)
        .collect();

    Node::Document { children }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // ============================================================
    // 빈 문서 테스트
    // ============================================================
    #[test]
    fn parse_empty_string() {
        let doc = parse("");
        assert_eq!(doc.children().len(), 0);
    }

    // ============================================================
    // Paragraph 테스트 (단일/여러 블록)
    // ============================================================
    #[rstest]
    #[case("hello", &["hello"])]
    #[case("\n\nparagraph", &["paragraph"])]           // 앞 빈 줄
    #[case("paragraph\n\n", &["paragraph"])]           // 뒤 빈 줄
    #[case("first\n\nsecond", &["first", "second"])]
    #[case("first\n\n\nsecond", &["first", "second"])] // 연속 빈 줄
    fn test_paragraph(#[case] input: &str, #[case] expected: &[&str]) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), expected.len(), "입력: {}", input);
        for (i, text) in expected.iter().enumerate() {
            assert_eq!(doc.children()[i].children()[0].as_text(), *text, "입력: {}", input);
        }
    }

    // ============================================================
    // ATX Heading 테스트 (입력, 레벨, 텍스트)
    // level = None이면 Paragraph, Some(n)이면 Heading
    // ============================================================
    #[rstest]
    // 기본 heading
    #[case("# heading", Some(1), "heading")]
    #[case("## heading", Some(2), "heading")]
    #[case("###### h6 title", Some(6), "h6 title")]
    // 빈 heading
    #[case("#", Some(1), "")]
    #[case("# ", Some(1), "")]
    #[case("### ###", Some(3), "")]                       // 닫는 #만
    // 닫는 # 시퀀스
    #[case("## foo ##", Some(2), "foo")]
    #[case("# foo ##########", Some(1), "foo")]           // 개수 불일치 OK
    #[case("### foo ###   ", Some(3), "foo")]             // 뒤 공백
    #[case("# foo#", Some(1), "foo#")]                    // 앞 공백 없음 → 텍스트
    #[case("### foo ### b", Some(3), "foo ### b")]        // 뒤에 문자 → 텍스트
    #[case("## a ## b", Some(2), "a ## b")]               // 중간 # → 텍스트
    // 탭 처리
    #[case("#\tfoo", Some(1), "foo")]                     // # 뒤 탭
    #[case("# foo\t#", Some(1), "foo")]                   // 닫는 # 앞 탭
    // 선행 공백 (0~3칸 허용)
    #[case(" # foo", Some(1), "foo")]
    #[case("   # foo", Some(1), "foo")]
    // # 뒤 여러 공백
    #[case("#    foo", Some(1), "foo")]
    // 내부 공백 유지
    #[case("# foo   bar", Some(1), "foo   bar")]
    // 유니코드
    #[case("# 안녕하세요", Some(1), "안녕하세요")]
    #[case("## 🎉 축하합니다", Some(2), "🎉 축하합니다")]
    // Heading이 아닌 케이스 (Paragraph로 처리)
    #[case("#no_space", None, "#no_space")]               // # 뒤 공백 없음
    #[case("####### not heading", None, "####### not heading")]  // 7개 이상 #
    #[case("    # foo", None, "# foo")]                   // 4칸 들여쓰기
    #[case("\t# foo", None, "# foo")]                     // 탭 = 4칸
    #[case("  \t# foo", None, "# foo")]                   // 2칸 + 탭 = 6칸
    fn test_heading(#[case] input: &str, #[case] level: Option<u8>, #[case] text: &str) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "입력: {}", input);
        if let Some(lvl) = level {
            assert_eq!(doc.children()[0].level(), lvl, "입력: {}", input);
        }
        assert_eq!(doc.children()[0].children()[0].as_text(), text, "입력: {}", input);
    }

    // ============================================================
    // Thematic Break 테스트
    // is_break = true면 ThematicBreak, false면 Paragraph
    // ============================================================
    #[rstest]
    // 기본 케이스 (3개)
    #[case("***", true)]
    #[case("---", true)]
    #[case("___", true)]
    // 3개 이상
    #[case("*****", true)]
    #[case("----------", true)]
    // 문자 사이 공백
    #[case("* * *", true)]
    #[case("- - -", true)]
    #[case("_  _  _", true)]                         // 여러 공백
    // 선행 공백 (0~3칸)
    #[case(" ***", true)]
    #[case("  ---", true)]
    #[case("   ___", true)]
    // 끝 공백
    #[case("***   ", true)]
    // Thematic Break가 아닌 케이스
    #[case("**", false)]                             // 2개 부족
    #[case("--", false)]
    #[case("__", false)]
    #[case("    ***", false)]                        // 4칸 들여쓰기
    #[case("*-*", false)]                            // 혼합 문자
    #[case("***a", false)]                           // 다른 문자 포함
    #[case("a]***", false)]                          // 앞에 다른 문자
    fn test_thematic_break(#[case] input: &str, #[case] is_break: bool) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "입력: {}", input);
        assert_eq!(doc.children()[0].is_thematic_break(), is_break, "입력: {}", input);
    }

    // ============================================================
    // Blockquote 테스트 (depth = None이면 Paragraph, Some(n)이면 n단계 중첩)
    // ============================================================
    #[rstest]
    // 단순 케이스 (depth = 1)
    #[case("> hello", Some(1), "hello")]
    #[case(">hello", Some(1), "hello")]                   // 공백 없어도 OK
    #[case(">  hello", Some(1), "hello")]                 // 여러 공백
    #[case(" > hello", Some(1), "hello")]                 // 선행 공백 1칸
    #[case("  > hello", Some(1), "hello")]                // 선행 공백 2칸
    #[case("   > hello", Some(1), "hello")]               // 선행 공백 3칸
    #[case("> 안녕하세요", Some(1), "안녕하세요")]        // 유니코드
    // 중첩 케이스
    #[case("> > nested", Some(2), "nested")]
    #[case("> > > deep", Some(3), "deep")]
    #[case("> > > > 4단계", Some(4), "4단계")]
    // Blockquote가 아닌 케이스
    #[case("    > hello", None, "> hello")]               // 4칸 들여쓰기 → Paragraph
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
}
