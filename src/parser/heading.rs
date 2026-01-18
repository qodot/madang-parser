//! ATX Heading 파싱
//!
//! CommonMark 명세: https://spec.commonmark.org/0.31.2/#atx-headings

use crate::node::Node;
use super::helpers::count_leading_char;

/// ATX Heading 파싱 시도
/// 성공하면 Some(Node::Heading), 실패하면 None
pub fn parse(trimmed: &str, indent: usize) -> Option<Node> {
    // 들여쓰기 3칸 초과면 Heading 아님
    if indent > 3 {
        return None;
    }

    // #로 시작하지 않으면 Heading 아님
    if !trimmed.starts_with('#') {
        return None;
    }

    // # 개수 세기
    let level = count_leading_char(trimmed, '#');

    // 레벨 1~6만 유효
    if level < 1 || level > 6 {
        return None;
    }

    let rest = &trimmed[level..];

    // # 뒤에 공백/탭이 있거나 빈 제목이어야 Heading
    if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
        let content = rest.trim();
        let content = strip_closing_hashes(content);
        Some(Node::Heading {
            level: level as u8,
            children: vec![Node::Text(content.to_string())],
        })
    } else {
        None
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
    use crate::parser::parse;
    use rstest::rstest;

    // level = None이면 Paragraph, Some(n)이면 Heading
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
    // 4칸 이상 들여쓰기 → Indented Code Block (별도 테스트)
    fn test_heading(#[case] input: &str, #[case] level: Option<u8>, #[case] text: &str) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "입력: {}", input);
        if let Some(lvl) = level {
            assert_eq!(doc.children()[0].level(), lvl, "입력: {}", input);
        }
        assert_eq!(doc.children()[0].children()[0].as_text(), text, "입력: {}", input);
    }
}
