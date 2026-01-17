//! List 파싱 테스트
//!
//! Bullet List와 Ordered List 파싱 테스트를 포함합니다.

#[cfg(test)]
mod tests {
    use crate::node::ListType;
    use crate::parser::parse;
    use rstest::rstest;

    // === Bullet List 테스트 ===

    /// 단일 아이템 Bullet List
    #[rstest]
    #[case("- item", 1, "item")]
    #[case("+ item", 1, "item")]
    #[case("* item", 1, "item")]
    fn single_bullet_list_item(#[case] input: &str, #[case] item_count: usize, #[case] text: &str) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "문서에 List가 하나여야 함");

        let list = &doc.children()[0];
        assert!(list.is_list(), "List여야 함: {:?}", list);
        assert_eq!(list.children().len(), item_count, "아이템 수");

        let item = &list.children()[0];
        assert!(item.is_list_item(), "ListItem이어야 함");

        // ListItem 안에 Paragraph가 있고, 그 안에 Text가 있음
        let para = &item.children()[0];
        assert_eq!(para.children()[0].as_text(), text);
    }

    /// 여러 아이템 tight Bullet List
    #[rstest]
    #[case("- a\n- b", 2, &["a", "b"])]
    #[case("- a\n- b\n- c", 3, &["a", "b", "c"])]
    fn multi_item_bullet_list(
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

    // === Ordered List 테스트 ===

    /// 단일 아이템 Ordered List
    #[rstest]
    #[case("1. item", '.', 1, "item")]
    #[case("1) item", ')', 1, "item")]
    #[case("5. item", '.', 5, "item")]
    #[case("10. item", '.', 10, "item")]
    fn single_ordered_list_item(
        #[case] input: &str,
        #[case] delimiter: char,
        #[case] start: usize,
        #[case] text: &str,
    ) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "문서에 List가 하나여야 함");

        let list = &doc.children()[0];
        assert!(list.is_list(), "List여야 함: {:?}", list);
        assert_eq!(
            *list.list_type(),
            ListType::Ordered { delimiter },
            "Ordered List여야 함"
        );
        assert_eq!(list.list_start(), start, "시작 번호");

        let item = &list.children()[0];
        let para = &item.children()[0];
        assert_eq!(para.children()[0].as_text(), text);
    }

    /// 여러 아이템 tight Ordered List
    #[rstest]
    #[case("1. a\n2. b", 2, &["a", "b"])]
    #[case("1. a\n2. b\n3. c", 3, &["a", "b", "c"])]
    fn multi_item_ordered_list(
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
}
