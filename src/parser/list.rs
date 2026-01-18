//! List 파싱 테스트
//!
//! Bullet List와 Ordered List 파싱 테스트를 포함합니다.

#[cfg(test)]
mod tests {
    use crate::node::{ListType, Node};
    use crate::parser::parse;
    use rstest::rstest;

    // AST 빌더 매크로
    macro_rules! text {
        ($s:expr) => {
            Node::Text($s.to_string())
        };
    }

    macro_rules! para {
        ($($child:expr),* $(,)?) => {
            Node::Paragraph { children: vec![$($child),*] }
        };
    }

    macro_rules! item {
        ($($child:expr),* $(,)?) => {
            Node::ListItem { children: vec![$($child),*] }
        };
    }

    macro_rules! list {
        ($($child:expr),* $(,)?) => {
            Node::List {
                list_type: ListType::Bullet,
                start: 1,
                tight: true,
                children: vec![$($child),*],
            }
        };
    }


    // CommonMark Example 261: 마커 뒤 공백 없으면 리스트 아님
    #[rstest]
    #[case("-one")]
    #[case("2.two")]
    fn test_example_261_not_list_marker(#[case] input: &str) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1);
        assert!(!doc.children()[0].is_list(), "List가 아니어야 함: {:?}", input);
    }

    /// 단일 아이템 List 테스트
    /// ordered = None이면 Bullet, Some((delimiter, start))이면 Ordered
    #[rstest]
    // Bullet List: 마커(-,+,*) 뒤 공백 필수
    #[case("- item", None, "item")]
    #[case("+ item", None, "item")]
    #[case("* item", None, "item")]
    // Ordered List
    #[case("1. item", Some(('.', 1)), "item")]
    #[case("1) item", Some((')', 1)), "item")]
    #[case("5. item", Some(('.', 5)), "item")]
    #[case("10. item", Some(('.', 10)), "item")]
    fn test_single_item_list(
        #[case] input: &str,
        #[case] ordered: Option<(char, usize)>,
        #[case] text: &str,
    ) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "문서에 List가 하나여야 함");

        let list = &doc.children()[0];
        assert!(list.is_list(), "List여야 함: {:?}", list);
        assert_eq!(list.children().len(), 1, "아이템 수");

        if let Some((delimiter, start)) = ordered {
            assert_eq!(
                *list.list_type(),
                ListType::Ordered { delimiter },
                "Ordered List여야 함"
            );
            assert_eq!(list.list_start(), start, "시작 번호");
        }

        let item = &list.children()[0];
        assert!(item.is_list_item(), "ListItem이어야 함");

        let para = &item.children()[0];
        assert_eq!(para.children()[0].as_text(), text);
    }

    /// 여러 아이템 tight List 테스트 (Bullet + Ordered)
    #[rstest]
    // Bullet List
    #[case("- a\n- b", 2, &["a", "b"])]
    #[case("- a\n- b\n- c", 3, &["a", "b", "c"])]
    // Ordered List
    #[case("1. a\n2. b", 2, &["a", "b"])]
    #[case("1. a\n2. b\n3. c", 3, &["a", "b", "c"])]
    fn test_multi_item_tight_list(
        #[case] input: &str,
        #[case] item_count: usize,
        #[case] texts: &[&str],
    ) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "문서에 List가 하나여야 함");

        let list = &doc.children()[0];
        assert!(list.is_list(), "List여야 함");
        assert!(list.is_tight(), "tight List여야 함");
        assert_eq!(list.children().len(), item_count, "아이템 수");

        for (i, text) in texts.iter().enumerate() {
            let item = &list.children()[i];
            let para = &item.children()[0];
            assert_eq!(para.children()[0].as_text(), *text, "아이템 {}", i);
        }
    }

    /// 아이템 간 빈 줄이 있는 Loose List
    #[rstest]
    #[case("- a\n\n- b", 2, &["a", "b"])]
    #[case("- a\n\n- b\n\n- c", 3, &["a", "b", "c"])]
    #[case("1. a\n\n2. b", 2, &["a", "b"])]
    fn loose_list(#[case] input: &str, #[case] item_count: usize, #[case] texts: &[&str]) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "문서에 List가 하나여야 함");

        let list = &doc.children()[0];
        assert!(list.is_list(), "List여야 함");
        assert!(!list.is_tight(), "loose List여야 함 (tight=false)");
        assert_eq!(list.children().len(), item_count, "아이템 수");

        for (i, text) in texts.iter().enumerate() {
            let item = &list.children()[i];
            let para = &item.children()[0];
            assert_eq!(para.children()[0].as_text(), *text, "아이템 {}", i);
        }
    }

    /// 다른 마커 타입은 별도 리스트로 분리 (Bullet + Ordered)
    #[rstest]
    // 다른 Bullet 마커
    #[case("- a\n+ b", &["a"], &["b"])]
    #[case("- a\n* b", &["a"], &["b"])]
    #[case("+ a\n- b", &["a"], &["b"])]
    #[case("- a\n- b\n+ c", &["a", "b"], &["c"])]
    // 다른 Ordered 구분자
    #[case("1. a\n1) b", &["a"], &["b"])]
    #[case("1) a\n1. b", &["a"], &["b"])]
    fn test_different_markers_create_separate_lists(
        #[case] input: &str,
        #[case] first_list_texts: &[&str],
        #[case] second_list_texts: &[&str],
    ) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 2, "문서에 List가 2개여야 함");

        let list1 = &doc.children()[0];
        assert!(list1.is_list(), "첫 번째가 List여야 함");
        assert_eq!(list1.children().len(), first_list_texts.len(), "첫 번째 리스트 아이템 수");
        for (i, text) in first_list_texts.iter().enumerate() {
            let item = &list1.children()[i];
            let para = &item.children()[0];
            assert_eq!(para.children()[0].as_text(), *text, "첫 번째 리스트 아이템 {}", i);
        }

        let list2 = &doc.children()[1];
        assert!(list2.is_list(), "두 번째가 List여야 함");
        assert_eq!(list2.children().len(), second_list_texts.len(), "두 번째 리스트 아이템 수");
        for (i, text) in second_list_texts.iter().enumerate() {
            let item = &list2.children()[i];
            let para = &item.children()[0];
            assert_eq!(para.children()[0].as_text(), *text, "두 번째 리스트 아이템 {}", i);
        }
    }

    /// 리스트 파싱 통합 테스트 (AST 비교)
    #[rstest]
    // 다중 라인 아이템 (continuation line)
    #[case("- line1\n  line2",
        list![item![para![text!["line1\nline2"]]]])]
    #[case("- line1\n  line2\n  line3",
        list![item![para![text!["line1\nline2\nline3"]]]])]
    // 다중 Paragraph 아이템 (빈 줄로 분리, 아이템 내부 빈 줄은 tight에 영향 없음)
    #[case("- foo\n\n  bar",
        list![item![para![text!["foo"]], para![text!["bar"]]]])]
    #[case("- foo\n\n\n  bar",
        list![item![para![text!["foo"]], para![text!["bar"]]]])]
    // 중첩 리스트
    #[case("- foo\n  - bar",
        list![item![para![text!["foo"]], list![item![para![text!["bar"]]]]]])]
    #[case("- foo\n  - bar\n  - baz",
        list![item![para![text!["foo"]], list![item![para![text!["bar"]]], item![para![text!["baz"]]]]]])]
    #[case("- foo\n  - bar\n- qux",
        list![item![para![text!["foo"]], list![item![para![text!["bar"]]]]], item![para![text!["qux"]]]])]
    fn list_structure(#[case] input: &str, #[case] expected: Node) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "문서에 List가 하나여야 함");
        assert_eq!(doc.children()[0], expected);
    }
}
