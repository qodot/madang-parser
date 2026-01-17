# 학습 진행 상황

## 마지막 학습일: 2026.1.18

## 이전 학습 요약 (PROGRESS-004)
- Fenced Code Block 전체 구현 (27개 테스트)
- 파서 구조: `split("\n\n")` → 라인 단위 스캔 + fold 패턴으로 변경
- 총 106개 테스트

## 오늘 완료된 내용

### 1. 공통 헬퍼 모듈 생성 (helpers.rs)

중복 코드를 helpers.rs로 추출:

```rust
//! 파서 공통 헬퍼 함수

/// 문자열 앞에서 특정 문자가 연속으로 몇 개 있는지 세기
pub(crate) fn count_leading_char(s: &str, c: char) -> usize {
    s.chars().take_while(|&ch| ch == c).count()
}

/// 들여쓰기 계산 (공백=1, 탭=4)
pub(crate) fn calculate_indent(s: &str) -> usize {
    s.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

/// 문자열에서 최대 n칸의 공백 제거
pub(crate) fn remove_indent(s: &str, n: usize) -> &str {
    let spaces = count_leading_char(s, ' ');
    let remove = spaces.min(n);
    &s[remove..]
}
```

### 2. fenced_code_block 리팩토링

- `try_start`: 펜스 시작 감지 (fence_char, fence_len, info, indent 반환)
- `is_end`: 닫는 펜스 확인
- 중복 함수 제거, helpers 모듈 활용
- 37개 테스트 추가 (try_start 20개, is_end 17개)

### 3. heading.rs 헬퍼 활용

```rust
// Before
let level = trimmed.chars().take_while(|c| *c == '#').count();

// After
let level = count_leading_char(trimmed, '#');
```

### 4. mod.rs 정리

- 중복 함수 제거 (`count_leading_char`, `count_leading_spaces`, `remove_indent`, `calculate_indent`)
- helpers 및 fenced_code_block 모듈에서 import

## 오늘 배운 러스트 개념

### pub(crate) 가시성
```rust
pub(crate) fn helper() { }  // 크레이트 내부에서만 공개
```

가시성 레벨:
- `fn` (기본): 해당 모듈 내에서만
- `pub(super)`: 부모 모듈까지
- `pub(crate)`: 크레이트 전체
- `pub`: 외부 크레이트까지

### 헬퍼 함수 추출 기준

**추출 O:**
- 여러 모듈에서 사용
- 일반적인 유틸리티 성격
- 의도를 명확히 전달

**추출 X:**
- 특정 모듈 전용 로직 (예: `strip_closing_hashes`)
- 너무 단순한 함수 (예: `push_node`)

### count_leading_char vs calculate_indent

두 함수는 비슷해 보이지만 다른 목적:
```rust
// count_leading_char: 한 종류 문자만
" \t code" → count_leading_char(s, ' ') = 1

// calculate_indent: 혼합 + 가중치
" \t code" → 1 + 4 = 5 (공백 1칸 + 탭 4칸)
```

## 현재 코드 구조

### helpers.rs 함수들
| 함수 | 설명 |
|------|------|
| `count_leading_char(s, c)` | 특정 문자 연속 개수 |
| `calculate_indent(s)` | 들여쓰기 계산 (탭=4) |
| `remove_indent(s, n)` | 최대 n칸 공백 제거 |

### 파서 모듈 구조
```
src/parser/
├── mod.rs               # 메인 파서 (fold 패턴)
├── helpers.rs           # 공통 헬퍼 (신규)
├── fenced_code_block.rs # Fenced Code Block
├── heading.rs           # ATX Heading
├── thematic_break.rs    # Thematic Break
├── blockquote.rs        # Blockquote
└── paragraph.rs         # Paragraph
```

## 테스트 현황
- **총 193개 테스트 통과**
- helpers: 48개 (count_leading_char 16, remove_indent 17, calculate_indent 15)
- fenced_code_block: 64개 (기존 27 + try_start 20 + is_end 17)
- 기타: 81개

## 커밋 히스토리

```
6712dea calculate_indent를 helpers로 이동
ae74da6 heading: count_leading_char 헬퍼 활용
a7a459d 공통 헬퍼 추출 및 fenced_code_block 리팩토링
```

## 다음 학습 시 시작점

1. **Indented Code Block**: 4칸 들여쓰기 코드 블록
2. **Setext Heading**: `===` 또는 `---` 스타일 제목
3. **List**: 순서 있는/없는 목록 (가장 복잡)
