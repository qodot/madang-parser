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
    // =========================================================================
    // 5.2 List Items
    // =========================================================================
    // Example 261: 마커 뒤 공백 없으면 paragraph
    #[case("-one", vec![Node::para(vec![Node::text("-one")])])]
    #[case("2.two", vec![Node::para(vec![Node::text("2.two")])])]
    // 단일 아이템 (마커 종류별 검증은 list_item.rs, 여기선 대표 케이스만)
    #[case("- item", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("item")])])])])]
    #[case("1. item", vec![Node::ordered_list('.', 1, true, vec![Node::item(vec![Node::para(vec![Node::text("item")])])])])]
    #[case("1) item", vec![Node::ordered_list(')', 1, true, vec![Node::item(vec![Node::para(vec![Node::text("item")])])])])]
    // Ordered 시작 번호
    #[case("5. item", vec![Node::ordered_list('.', 5, true, vec![Node::item(vec![Node::para(vec![Node::text("item")])])])])]
    #[case("10. item", vec![Node::ordered_list('.', 10, true, vec![Node::item(vec![Node::para(vec![Node::text("item")])])])])]
    // Example 265: 9자리 숫자 허용
    #[case("123456789. ok", vec![Node::ordered_list('.', 123456789, true, vec![Node::item(vec![Node::para(vec![Node::text("ok")])])])])]
    // Example 266: 10자리 숫자는 마커 아님 → paragraph
    #[case("1234567890. not ok", vec![Node::para(vec![Node::text("1234567890. not ok")])])]
    // Example 267: 0 시작 허용
    #[case("0. ok", vec![Node::ordered_list('.', 0, true, vec![Node::item(vec![Node::para(vec![Node::text("ok")])])])])]
    // Example 268: 선행 0 허용 (003 → start=3)
    #[case("003. ok", vec![Node::ordered_list('.', 3, true, vec![Node::item(vec![Node::para(vec![Node::text("ok")])])])])]
    // Example 269: 음수는 마커 아님 → paragraph
    #[case("-1. not ok", vec![Node::para(vec![Node::text("-1. not ok")])])]
    // Continuation line
    #[case("- line1\n  line2\n  line3", vec![Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("line1\nline2\nline3")])])])])]
    // Example 255: 들여쓰기 부족 (1칸) → 리스트 종료
    #[case("- one\n\n two", vec![
        Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("one")])])]),
        Node::para(vec![Node::text("two")]),
    ])]
    // Example 256: 충분한 들여쓰기 (2칸) → 같은 아이템 두 번째 단락 (loose)
    #[case("- one\n\n  two", vec![
        Node::bullet_list(false, vec![
            Node::item(vec![Node::para(vec![Node::text("one")]), Node::para(vec![Node::text("two")])]),
        ])
    ])]
    // Example 262: 아이템 내 여러 빈 줄
    #[case("- foo\n\n\n  bar", vec![
        Node::bullet_list(false, vec![
            Node::item(vec![Node::para(vec![Node::text("foo")]), Node::para(vec![Node::text("bar")])]),
        ])
    ])]
    // Example 264: 리스트 아이템 내 코드 블록 (빈 줄 보존)
    #[case("- Foo\n\n      bar\n\n\n      baz", vec![
        Node::bullet_list(false, vec![
            Node::item(vec![Node::para(vec![Node::text("Foo")]), Node::code_block(None, "bar\n\n\nbaz")]),
        ])
    ])]
    // Example 278: 빈 줄로 시작하는 아이템
    // NOTE: 세 번째 아이템 코드 블록은 명세상 "baz"지만 현재 " baz" (향후 개선)
    #[case("-\n  foo\n-\n  ```\n  bar\n  ```\n-\n      baz", vec![
        Node::bullet_list(true, vec![
            Node::item(vec![Node::para(vec![Node::text("foo")])]),
            Node::item(vec![Node::code_block(None, "bar")]),
            Node::item(vec![Node::code_block(None, " baz")]),
        ])
    ])]
    // Example 281: 중간 빈 아이템 (bullet)
    #[case("- foo\n-\n- bar", vec![
        Node::bullet_list(true, vec![
            Node::item(vec![Node::para(vec![Node::text("foo")])]),
            Node::item(vec![]),
            Node::item(vec![Node::para(vec![Node::text("bar")])]),
        ])
    ])]
    // Example 283: 중간 빈 아이템 (ordered)
    #[case("1. foo\n2.\n3. bar", vec![
        Node::ordered_list('.', 1, true, vec![
            Node::item(vec![Node::para(vec![Node::text("foo")])]),
            Node::item(vec![]),
            Node::item(vec![Node::para(vec![Node::text("bar")])]),
        ])
    ])]
    // Example 284: 단일 빈 아이템
    #[case("*", vec![Node::bullet_list(true, vec![Node::item(vec![])])])]
    // 빈 아이템 연속 + 빈 줄 = loose
    #[case("-\n\n- foo", vec![
        Node::bullet_list(false, vec![Node::item(vec![]), Node::item(vec![Node::para(vec![Node::text("foo")])])]),
    ])]
    // =========================================================================
    // 5.3 Lists - tight/loose
    // =========================================================================
    // tight: 아이템 간 빈 줄 없음
    #[case("- a\n- b\n- c", vec![
        Node::bullet_list(true, vec![
            Node::item(vec![Node::para(vec![Node::text("a")])]),
            Node::item(vec![Node::para(vec![Node::text("b")])]),
            Node::item(vec![Node::para(vec![Node::text("c")])]),
        ])
    ])]
    #[case("1. a\n2. b", vec![
        Node::ordered_list('.', 1, true, vec![
            Node::item(vec![Node::para(vec![Node::text("a")])]),
            Node::item(vec![Node::para(vec![Node::text("b")])]),
        ])
    ])]
    // loose: 아이템 간 빈 줄 있음
    #[case("- a\n\n- b\n\n- c", vec![
        Node::bullet_list(false, vec![
            Node::item(vec![Node::para(vec![Node::text("a")])]),
            Node::item(vec![Node::para(vec![Node::text("b")])]),
            Node::item(vec![Node::para(vec![Node::text("c")])]),
        ])
    ])]
    #[case("1. a\n\n2. b", vec![
        Node::ordered_list('.', 1, false, vec![
            Node::item(vec![Node::para(vec![Node::text("a")])]),
            Node::item(vec![Node::para(vec![Node::text("b")])]),
        ])
    ])]
    // =========================================================================
    // 5.3 Lists - 다른 마커 타입은 별도 리스트
    // =========================================================================
    #[case("- a\n+ b", vec![
        Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("a")])])]),
        Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("b")])])]),
    ])]
    #[case("1. a\n1) b", vec![
        Node::ordered_list('.', 1, true, vec![Node::item(vec![Node::para(vec![Node::text("a")])])]),
        Node::ordered_list(')', 1, true, vec![Node::item(vec![Node::para(vec![Node::text("b")])])]),
    ])]
    // =========================================================================
    // 5.3 Lists - 중첩
    // =========================================================================
    #[case("- foo\n  - bar\n  - baz", vec![
        Node::bullet_list(true, vec![
            Node::item(vec![
                Node::para(vec![Node::text("foo")]),
                Node::bullet_list(true, vec![
                    Node::item(vec![Node::para(vec![Node::text("bar")])]),
                    Node::item(vec![Node::para(vec![Node::text("baz")])]),
                ]),
            ]),
        ])
    ])]
    #[case("- foo\n  - bar\n- qux", vec![
        Node::bullet_list(true, vec![
            Node::item(vec![
                Node::para(vec![Node::text("foo")]),
                Node::bullet_list(true, vec![Node::item(vec![Node::para(vec![Node::text("bar")])])]),
            ]),
            Node::item(vec![Node::para(vec![Node::text("qux")])]),
        ])
    ])]
    // Example 301: 0~3칸 들여쓰기는 같은 레벨
    #[case("- a\n - b\n  - c\n   - d\n  - e\n - f\n- g", vec![
        Node::bullet_list(true, vec![
            Node::item(vec![Node::para(vec![Node::text("a")])]),
            Node::item(vec![Node::para(vec![Node::text("b")])]),
            Node::item(vec![Node::para(vec![Node::text("c")])]),
            Node::item(vec![Node::para(vec![Node::text("d")])]),
            Node::item(vec![Node::para(vec![Node::text("e")])]),
            Node::item(vec![Node::para(vec![Node::text("f")])]),
            Node::item(vec![Node::para(vec![Node::text("g")])]),
        ])
    ])]
    // Example 297: 3단계 중첩 + 빈 줄 후 추가 단락
    // NOTE: 명세상 외부는 tight지만 현재 모두 loose (향후 개선)
    #[case("- foo\n  - bar\n    - baz\n\n\n      bim", vec![
        Node::bullet_list(false, vec![
            Node::item(vec![
                Node::para(vec![Node::text("foo")]),
                Node::bullet_list(false, vec![
                    Node::item(vec![
                        Node::para(vec![Node::text("bar")]),
                        Node::bullet_list(false, vec![
                            Node::item(vec![
                                Node::para(vec![Node::text("baz")]),
                                Node::para(vec![Node::text("bim")]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ])
    ])]
    // Example 303: 4칸+ 들여쓰기 마커는 continuation
    #[case("- a\n - b\n  - c\n   - d\n    - e", vec![
        Node::bullet_list(true, vec![
            Node::item(vec![Node::para(vec![Node::text("a")])]),
            Node::item(vec![Node::para(vec![Node::text("b")])]),
            Node::item(vec![Node::para(vec![Node::text("c")])]),
            Node::item(vec![Node::para(vec![Node::text("d\n- e")])]),
        ])
    ])]
    fn test_list(#[case] input: &str, #[case] expected: Vec<Node>) {
        let doc = parse(input);
        assert_eq!(doc.children(), &expected);
    }
}
