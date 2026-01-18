//! List 파싱 통합 테스트
//!
//! CommonMark 명세 기준:
//! - 5.2 List Items: 마커 인식, continuation line → list_item.rs에서 유닛 테스트
//! - 5.3 Lists: tight/loose, 마커 타입별 분리, 중첩 → 이 파일에서 통합 테스트

#[cfg(test)]
mod tests {
    use crate::node::Node;
    use crate::parser::parse;
    use rstest::rstest;

    #[rstest]
    // 5.2 List Items - 통합 검증
    // Example 261: 마커 뒤 공백 없으면 paragraph (유닛 테스트는 list_item.rs)
    #[case("-one", vec![Node::para(vec![Node::text("-one")])])]
    #[case("2.two", vec![Node::para(vec![Node::text("2.two")])])]
    // 단일 아이템
    #[case("- item", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("item")])])])])]
    #[case("+ item", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("item")])])])])]
    #[case("* item", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("item")])])])])]
    #[case("1. item", vec![Node::ordered_list('.', 1, true, vec![Node::item(vec![Node::para(vec![Node::text("item")])])])])]
    #[case("1) item", vec![Node::ordered_list(')', 1, true, vec![Node::item(vec![Node::para(vec![Node::text("item")])])])])]
    #[case("5. item", vec![Node::ordered_list('.', 5, true, vec![Node::item(vec![Node::para(vec![Node::text("item")])])])])]
    #[case("10. item", vec![Node::ordered_list('.', 10, true, vec![Node::item(vec![Node::para(vec![Node::text("item")])])])])]
    // Continuation line
    #[case("- line1\n  line2", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("line1\nline2")])])])])]
    #[case("- line1\n  line2\n  line3", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("line1\nline2\nline3")])])])])]
    // 다중 블록 아이템 (빈 줄로 분리)
    #[case("- foo\n\n  bar", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("foo")]), Node::para(vec![Node::text("bar")])])])])]
    // 5.3 Lists - tight/loose
    #[case("- a\n- b", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("a")])]), Node::item(vec![Node::para(vec![Node::text("b")])])])])]
    #[case("- a\n- b\n- c", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("a")])]), Node::item(vec![Node::para(vec![Node::text("b")])]), Node::item(vec![Node::para(vec![Node::text("c")])])])])]
    #[case("1. a\n2. b", vec![Node::ordered_list('.', 1, true, vec![Node::item(vec![Node::para(vec![Node::text("a")])]), Node::item(vec![Node::para(vec![Node::text("b")])])])])]
    #[case("- a\n\n- b", vec![Node::bullet_list(false, vec![Node::item(vec![Node::para(vec![Node::text("a")])]), Node::item(vec![Node::para(vec![Node::text("b")])])])])]
    #[case("- a\n\n- b\n\n- c", vec![Node::bullet_list(false, vec![Node::item(vec![Node::para(vec![Node::text("a")])]), Node::item(vec![Node::para(vec![Node::text("b")])]), Node::item(vec![Node::para(vec![Node::text("c")])])])])]
    #[case("1. a\n\n2. b", vec![Node::ordered_list('.', 1, false, vec![Node::item(vec![Node::para(vec![Node::text("a")])]), Node::item(vec![Node::para(vec![Node::text("b")])])])])]
    // 5.3 Lists - 다른 마커 타입은 별도 리스트
    #[case("- a\n+ b", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("a")])])]), Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("b")])])])])]
    #[case("- a\n- b\n+ c", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("a")])]), Node::item(vec![Node::para(vec![Node::text("b")])])]), Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("c")])])])])]
    #[case("1. a\n1) b", vec![Node::ordered_list('.', 1, true, vec![Node::item(vec![Node::para(vec![Node::text("a")])])]), Node::ordered_list(')', 1, true, vec![Node::item(vec![Node::para(vec![Node::text("b")])])])])]
    // 5.3 Lists - 중첩
    #[case("- foo\n  - bar", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("foo")]), Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("bar")])])])])])])]
    #[case("- foo\n  - bar\n  - baz", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("foo")]), Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("bar")])]), Node::item(vec![Node::para(vec![Node::text("baz")])])])])])])]
    #[case("- foo\n  - bar\n- qux", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("foo")]), Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("bar")])])])]), Node::item(vec![Node::para(vec![Node::text("qux")])])])])]
    fn test_list(#[case] input: &str, #[case] expected: Vec<Node>) {
        let doc = parse(input);
        assert_eq!(doc.children(), &expected);
    }
}
