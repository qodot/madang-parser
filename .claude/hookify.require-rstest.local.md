---
name: require-rstest
enabled: true
event: file
conditions:
  - field: file_path
    operator: regex_match
    pattern: \.rs$
  - field: new_text
    operator: regex_match
    pattern: "#\\[test\\]"
action: block
---

**#[test] 사용 금지**

이 프로젝트는 `#[test]` 대신 `#[rstest]`를 사용합니다.

**수정 방법:**
```rust
// 잘못된 예
#[test]
fn test_something() { ... }

// 올바른 예
#[rstest]
#[case("input1", expected1)]
#[case("input2", expected2)]
fn test_something(#[case] input: &str, #[case] expected: Type) { ... }
```

rstest로 parametrized 테스트를 작성해주세요.
