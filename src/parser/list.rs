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

    // === Loose List 테스트 ===

    /// 아이템 간 빈 줄이 있는 Loose List
    #[rstest]
    #[case("- a\n\n- b", 2, &["a", "b"])]                    // 기본 loose
    #[case("- a\n\n- b\n\n- c", 3, &["a", "b", "c"])]        // 3개 아이템
    #[case("1. a\n\n2. b", 2, &["a", "b"])]                  // Ordered loose
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

    // === 다른 마커로 리스트 분리 테스트 ===

    /// 다른 Bullet 마커는 별도 리스트로 분리
    #[rstest]
    #[case("- a\n+ b", &["a"], &["b"])]           // dash → plus
    #[case("- a\n* b", &["a"], &["b"])]           // dash → asterisk
    #[case("+ a\n- b", &["a"], &["b"])]           // plus → dash
    #[case("- a\n- b\n+ c", &["a", "b"], &["c"])] // 2개 dash → 1개 plus
    fn different_bullet_markers_create_separate_lists(
        #[case] input: &str,
        #[case] first_list_texts: &[&str],
        #[case] second_list_texts: &[&str],
    ) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 2, "문서에 List가 2개여야 함");

        // 첫 번째 리스트
        let list1 = &doc.children()[0];
        assert!(list1.is_list(), "첫 번째가 List여야 함");
        assert_eq!(list1.children().len(), first_list_texts.len(), "첫 번째 리스트 아이템 수");
        for (i, text) in first_list_texts.iter().enumerate() {
            let item = &list1.children()[i];
            let para = &item.children()[0];
            assert_eq!(para.children()[0].as_text(), *text, "첫 번째 리스트 아이템 {}", i);
        }

        // 두 번째 리스트
        let list2 = &doc.children()[1];
        assert!(list2.is_list(), "두 번째가 List여야 함");
        assert_eq!(list2.children().len(), second_list_texts.len(), "두 번째 리스트 아이템 수");
        for (i, text) in second_list_texts.iter().enumerate() {
            let item = &list2.children()[i];
            let para = &item.children()[0];
            assert_eq!(para.children()[0].as_text(), *text, "두 번째 리스트 아이템 {}", i);
        }
    }

    /// 다른 Ordered 구분자는 별도 리스트로 분리
    #[rstest]
    #[case("1. a\n1) b", &["a"], &["b"])]  // dot → paren
    #[case("1) a\n1. b", &["a"], &["b"])]  // paren → dot
    fn different_ordered_delimiters_create_separate_lists(
        #[case] input: &str,
        #[case] first_list_texts: &[&str],
        #[case] second_list_texts: &[&str],
    ) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 2, "문서에 List가 2개여야 함");

        let list1 = &doc.children()[0];
        let list2 = &doc.children()[1];

        assert!(list1.is_list() && list2.is_list(), "둘 다 List여야 함");
        assert_eq!(list1.children().len(), first_list_texts.len());
        assert_eq!(list2.children().len(), second_list_texts.len());
    }

    // === 다중 라인 아이템 테스트 ===

    /// 다중 라인 아이템 (continuation line)
    /// content_indent 이상 들여쓰기된 줄은 같은 아이템에 속함
    /// 초과 들여쓰기는 내용의 일부로 유지됨
    /// 빈 줄도 내용에 포함됨 (CommonMark 명세 준수)
    #[rstest]
    #[case("- line1\n  line2", 1, "line1\nline2")]              // 정확히 2칸 들여쓰기
    #[case("- line1\n   line2", 1, "line1\n line2")]            // 3칸 → 초과 1칸 유지
    #[case("- line1\n  line2\n  line3", 1, "line1\nline2\nline3")] // 3줄 continuation
    #[case("- foo\n\n  bar", 1, "foo\n\nbar")]                  // 빈 줄 포함 (명세 준수)
    #[case("- foo\n\n\n  bar", 1, "foo\n\n\nbar")]              // 빈 줄 여러 개
    fn multi_line_item(#[case] input: &str, #[case] item_count: usize, #[case] expected_text: &str) {
        let doc = parse(input);
        assert_eq!(doc.children().len(), 1, "문서에 List가 하나여야 함");

        let list = &doc.children()[0];
        assert!(list.is_list(), "List여야 함: {:?}", list);
        assert_eq!(list.children().len(), item_count, "아이템 수");

        let item = &list.children()[0];
        let para = &item.children()[0];
        assert_eq!(para.children()[0].as_text(), expected_text, "다중 라인 텍스트");
    }
}
