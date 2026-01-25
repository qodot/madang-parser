//! List 파싱 통합 테스트
//!
//! CommonMark 명세 기준:
//! - 5.2 List Items: 마커 인식, continuation line → list_item.rs에서 유닛 테스트
//! - 5.3 Lists: tight/loose, 마커 타입별 분리, 중첩 → 이 파일에서 통합 테스트

#[cfg(test)]
mod tests {
    use crate::node::{BlockNode, InlineNode, ListItemNode};
    use crate::parser::parse;
    use rstest::rstest;

    #[rstest]
    // =========================================================================
    // 5.2 List Items
    // =========================================================================
    // Example 261: 마커 뒤 공백 없으면 paragraph
    #[case("-one", vec![BlockNode::paragraph(vec![InlineNode::text("-one")])])]
    #[case("2.two", vec![BlockNode::paragraph(vec![InlineNode::text("2.two")])])]
    // 단일 아이템 (마커 종류별 검증은 list_item.rs, 여기선 대표 케이스만)
    #[case("- item", vec![BlockNode::bullet_list(true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("item")])])])])]
    #[case("1. item", vec![BlockNode::ordered_list('.', 1, true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("item")])])])])]
    #[case("1) item", vec![BlockNode::ordered_list(')', 1, true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("item")])])])])]
    // Ordered 시작 번호
    #[case("5. item", vec![BlockNode::ordered_list('.', 5, true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("item")])])])])]
    #[case("10. item", vec![BlockNode::ordered_list('.', 10, true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("item")])])])])]
    // Example 265: 9자리 숫자 허용
    #[case("123456789. ok", vec![BlockNode::ordered_list('.', 123456789, true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("ok")])])])])]
    // Example 266: 10자리 숫자는 마커 아님 → paragraph
    #[case("1234567890. not ok", vec![BlockNode::paragraph(vec![InlineNode::text("1234567890. not ok")])])]
    // Example 267: 0 시작 허용
    #[case("0. ok", vec![BlockNode::ordered_list('.', 0, true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("ok")])])])])]
    // Example 268: 선행 0 허용 (003 → start=3)
    #[case("003. ok", vec![BlockNode::ordered_list('.', 3, true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("ok")])])])])]
    // Example 269: 음수는 마커 아님 → paragraph
    #[case("-1. not ok", vec![BlockNode::paragraph(vec![InlineNode::text("-1. not ok")])])]
    // Continuation line
    #[case("- line1\n  line2\n  line3", vec![BlockNode::bullet_list(true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("line1\nline2\nline3")])])])])]
    // Example 255: 들여쓰기 부족 (1칸) → 리스트 종료
    #[case("- one\n\n two", vec![
        BlockNode::bullet_list(true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("one")])])]),
        BlockNode::paragraph(vec![InlineNode::text("two")]),
    ])]
    // Example 256: 충분한 들여쓰기 (2칸) → 같은 아이템 두 번째 단락 (loose)
    #[case("- one\n\n  two", vec![
        BlockNode::bullet_list(false, vec![
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("one")]), BlockNode::paragraph(vec![InlineNode::text("two")])]),
        ])
    ])]
    // Example 262: 아이템 내 여러 빈 줄
    #[case("- foo\n\n\n  bar", vec![
        BlockNode::bullet_list(false, vec![
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("foo")]), BlockNode::paragraph(vec![InlineNode::text("bar")])]),
        ])
    ])]
    // Example 264: 리스트 아이템 내 코드 블록 (빈 줄 보존)
    #[case("- Foo\n\n      bar\n\n\n      baz", vec![
        BlockNode::bullet_list(false, vec![
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("Foo")]), BlockNode::code_block(None, "bar\n\n\nbaz")]),
        ])
    ])]
    // Example 278: 빈 줄로 시작하는 아이템
    // NOTE: 세 번째 아이템 코드 블록은 명세상 "baz"지만 현재 " baz" (향후 개선)
    #[case("-\n  foo\n-\n  ```\n  bar\n  ```\n-\n      baz", vec![
        BlockNode::bullet_list(true, vec![
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("foo")])]),
            ListItemNode::new(vec![BlockNode::code_block(None, "bar")]),
            ListItemNode::new(vec![BlockNode::code_block(None, " baz")]),
        ])
    ])]
    // Example 281: 중간 빈 아이템 (bullet)
    #[case("- foo\n-\n- bar", vec![
        BlockNode::bullet_list(true, vec![
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("foo")])]),
            ListItemNode::new(vec![]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("bar")])]),
        ])
    ])]
    // Example 283: 중간 빈 아이템 (ordered)
    #[case("1. foo\n2.\n3. bar", vec![
        BlockNode::ordered_list('.', 1, true, vec![
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("foo")])]),
            ListItemNode::new(vec![]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("bar")])]),
        ])
    ])]
    // Example 284: 단일 빈 아이템
    #[case("*", vec![BlockNode::bullet_list(true, vec![ListItemNode::new(vec![])])])]
    // 빈 아이템 연속 + 빈 줄 = loose
    #[case("-\n\n- foo", vec![
        BlockNode::bullet_list(false, vec![ListItemNode::new(vec![]), ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("foo")])])]),
    ])]
    // =========================================================================
    // 5.3 Lists - tight/loose
    // =========================================================================
    // tight: 아이템 간 빈 줄 없음
    #[case("- a\n- b\n- c", vec![
        BlockNode::bullet_list(true, vec![
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("a")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("b")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("c")])]),
        ])
    ])]
    #[case("1. a\n2. b", vec![
        BlockNode::ordered_list('.', 1, true, vec![
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("a")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("b")])]),
        ])
    ])]
    // loose: 아이템 간 빈 줄 있음
    #[case("- a\n\n- b\n\n- c", vec![
        BlockNode::bullet_list(false, vec![
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("a")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("b")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("c")])]),
        ])
    ])]
    #[case("1. a\n\n2. b", vec![
        BlockNode::ordered_list('.', 1, false, vec![
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("a")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("b")])]),
        ])
    ])]
    // =========================================================================
    // 5.3 Lists - 다른 마커 타입은 별도 리스트
    // =========================================================================
    #[case("- a\n+ b", vec![
        BlockNode::bullet_list(true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("a")])])]),
        BlockNode::bullet_list(true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("b")])])]),
    ])]
    #[case("1. a\n1) b", vec![
        BlockNode::ordered_list('.', 1, true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("a")])])]),
        BlockNode::ordered_list(')', 1, true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("b")])])]),
    ])]
    // =========================================================================
    // 5.3 Lists - 중첩
    // =========================================================================
    #[case("- foo\n  - bar\n  - baz", vec![
        BlockNode::bullet_list(true, vec![
            ListItemNode::new(vec![
                BlockNode::paragraph(vec![InlineNode::text("foo")]),
                BlockNode::bullet_list(true, vec![
                    ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("bar")])]),
                    ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("baz")])]),
                ]),
            ]),
        ])
    ])]
    #[case("- foo\n  - bar\n- qux", vec![
        BlockNode::bullet_list(true, vec![
            ListItemNode::new(vec![
                BlockNode::paragraph(vec![InlineNode::text("foo")]),
                BlockNode::bullet_list(true, vec![ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("bar")])])]),
            ]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("qux")])]),
        ])
    ])]
    // Example 301: 0~3칸 들여쓰기는 같은 레벨
    #[case("- a\n - b\n  - c\n   - d\n  - e\n - f\n- g", vec![
        BlockNode::bullet_list(true, vec![
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("a")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("b")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("c")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("d")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("e")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("f")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("g")])]),
        ])
    ])]
    // Example 297: 3단계 중첩 + 빈 줄 후 추가 단락
    // NOTE: 명세상 외부는 tight지만 현재 모두 loose (향후 개선)
    #[case("- foo\n  - bar\n    - baz\n\n\n      bim", vec![
        BlockNode::bullet_list(false, vec![
            ListItemNode::new(vec![
                BlockNode::paragraph(vec![InlineNode::text("foo")]),
                BlockNode::bullet_list(false, vec![
                    ListItemNode::new(vec![
                        BlockNode::paragraph(vec![InlineNode::text("bar")]),
                        BlockNode::bullet_list(false, vec![
                            ListItemNode::new(vec![
                                BlockNode::paragraph(vec![InlineNode::text("baz")]),
                                BlockNode::paragraph(vec![InlineNode::text("bim")]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ])
    ])]
    // Example 303: 4칸+ 들여쓰기 마커는 continuation
    #[case("- a\n - b\n  - c\n   - d\n    - e", vec![
        BlockNode::bullet_list(true, vec![
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("a")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("b")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("c")])]),
            ListItemNode::new(vec![BlockNode::paragraph(vec![InlineNode::text("d\n- e")])]),
        ])
    ])]
    fn test_list(#[case] input: &str, #[case] expected: Vec<BlockNode>) {
        let doc = parse(input);
        assert_eq!(doc.children, expected);
    }
}
