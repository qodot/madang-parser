# PROGRESS-009: Example 297 중첩 리스트 continuation 문제 해결

## 날짜
2026-01-20

## 이번 세션 요약

### 해결한 문제: Example 297

**입력:**
```markdown
- foo
  - bar
    - baz


      bim
```

**기대 결과:**
- `bim`이 가장 안쪽 리스트 아이템 `baz`의 두 번째 단락으로 파싱
- 3단계 중첩 리스트 구조 유지

**이전 문제:**
- `bim`이 `CodeBlock`으로 잘못 파싱됨
- 원인: 빈 줄로 청크 분리 후 별도 재파싱 시 4칸 들여쓰기가 CodeBlock으로 인식

### 문제 분석 과정

1. **데이터 흐름 추적**
   - 원본 `"      bim"` (6칸) → 외부 리스트 `content_indent=2` 만큼 제거 → `"    bim"` (4칸)
   - `parse_item_lines`에서 빈 줄 기준 청크 분리
   - 청크 1: `foo\n- bar\n  - baz`
   - 청크 2: `    bim` (별도 재파싱 → CodeBlock!)

2. **핵심 인사이트**
   - 빈 줄로 청크를 분리하면 리스트 컨텍스트 정보가 손실됨
   - 중첩 리스트에서 빈 줄 후 들여쓰기된 내용은 해당 레벨의 continuation이어야 함

### 해결책

**`parse_item_lines` 함수 수정:**

```rust
fn parse_item_lines(lines: &[ItemLine]) -> Vec<Node> {
    let has_any_text_only = lines.iter().any(|l| l.text_only);

    if has_any_text_only {
        // text_only가 있는 경우: 청크 단위로 처리 (기존 로직)
        parse_item_lines_with_text_only(lines)
    } else {
        // text_only가 없는 경우: 전체를 한 번에 재파싱
        // 빈 줄이 있어도 리스트 continuation으로 처리됨
        let content = lines.iter()
            .map(|l| l.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let doc = parse(&content);
        match doc {
            Node::Document { children } => children,
            _ => vec![doc],
        }
    }
}
```

**핵심 변경:**
- `text_only` 플래그가 없으면 빈 줄로 청크 분리하지 않음
- 전체 내용을 한 번에 재파싱하여 리스트 컨텍스트 유지

### 학습 포인트

1. **청크 분리의 함정**
   - 빈 줄로 무조건 분리하면 컨텍스트 손실
   - 중첩 구조에서는 특히 주의 필요

2. **조건부 처리의 중요성**
   - `text_only`가 있는 경우만 특별 처리 (Example 303)
   - 일반 케이스는 단순하게 유지

3. **재파싱의 원리**
   - 전체를 한 번에 재파싱하면 파서가 자연스럽게 리스트 continuation 처리
   - 파서의 기존 로직을 최대한 활용

## 현재 구현 상태

### 완료된 CommonMark Examples
- Example 261: 마커 뒤 공백 필수
- Example 265-269: 마커 인식 규칙
- Example 301: 0-3칸 들여쓰기는 같은 레벨 아이템
- Example 303: 4칸+ 들여쓰기 마커는 continuation text
- Example 297: 중첩 리스트 빈 줄 후 continuation

### 테스트 현황
- 총 429개 테스트 통과
- 리스트 관련 27개 테스트 케이스 모두 통과

## 주요 파일 변경

1. **`src/parser/mod.rs`**
   - `parse_item_lines`: 조건부 청크 분리 로직 추가
   - `parse_item_lines_with_text_only`: text_only용 별도 함수 분리

2. **`src/parser/list_item.rs`** (이전 세션)
   - `try_end`: `first_content_indent`와 `current_content_indent` 분리
   - `ItemLine::text_only()` 반환 로직

3. **`src/parser/context.rs`** (이전 세션)
   - `ItemLine` 구조체: `text_only` 플래그 추가
   - `ListContinueReason::ContinuationLine`: `ItemLine` 사용

## 다음 학습 제안

1. **더 많은 CommonMark List Examples 구현**
   - Example 264: 빈 아이템
   - Example 298-300: loose/tight 리스트 규칙

2. **HTML Block 파싱**
   - CommonMark 4.6 HTML Blocks

3. **Link Reference Definition**
   - CommonMark 4.7 Link reference definitions
