# 학습 진행 상황

## 마지막 학습일: 2026.1.17

## 이전 학습 요약 (PROGRESS-003)
- Paragraph, ATX Heading, Thematic Break, Blockquote 구현
- 총 79개 테스트

## 오늘 완료된 내용

### Fenced Code Block 전체 구현 (27개 테스트)

1. **기본 구조**
   - CodeBlock variant: `{ info: Option<String>, content: String }`
   - 백틱(```) 및 틸드(~~~) 펜스 지원

2. **info string 파싱**
   - ` ```rust ` → info = "rust"
   - 앞뒤 공백 자동 제거 (trim)

3. **펜스 길이 매칭**
   - 닫는 펜스 길이 ≥ 여는 펜스 길이
   - 같은 문자(백틱/틸드) 확인

4. **닫는 펜스 없음 처리 (CommonMark 명세)**
   - 유효한 닫는 펜스가 없으면 EOF까지 코드 블록
   - 잘못된 닫는 펜스(다른 문자, 짧은 길이)도 코드 내용에 포함

5. **들여쓰기 처리**
   - 여는 펜스 앞 0-3칸 허용 (4칸 이상은 펜스 아님)
   - 여는 펜스 들여쓰기만큼 내용의 각 줄에서 제거

6. **메인 파서 통합**
   - `parse_block`에서 fenced_code_block 우선 시도

## 오늘 배운 러스트 개념

### Iterator 체이닝으로 연속 문자 세기
```rust
fn count_leading_char(s: &str, c: char) -> usize {
    s.chars().take_while(|&ch| ch == c).count()
}
```

### 최대 n칸만 제거하는 헬퍼
```rust
fn remove_indent(s: &str, n: usize) -> &str {
    let spaces = count_leading_char(s, ' ');
    let remove = spaces.min(n);  // 초과 제거 방지
    &s[remove..]
}
```

### Option 체이닝 확장
```rust
fenced_code_block::parse(block, indent)
    .or_else(|| thematic_break::parse(trimmed, indent))
    .or_else(|| blockquote::parse(trimmed, indent, parse_block))
    .or_else(|| heading::parse(trimmed, indent))
    .unwrap_or_else(|| paragraph::parse(trimmed))
```

### rstest 테스트 통합 패턴
```rust
// Option<(&str, Option<&str>)>로 (content, info) 또는 None 검증
#[case("```rust\ncode\n```", Some(("code", Some("rust"))))]
#[case("code", None)]
```

## 현재 코드 구조

### Node 타입
```rust
pub enum Node {
    Document { children: Vec<Node> },
    Heading { level: u8, children: Vec<Node> },
    Paragraph { children: Vec<Node> },
    Blockquote { children: Vec<Node> },
    CodeBlock { info: Option<String>, content: String },
    ThematicBreak,
    Text(String),
}
```

### 파서 모듈 구조
```
src/parser/
├── mod.rs               # 디스패처
├── heading.rs           # ATX Heading
├── thematic_break.rs    # Thematic Break
├── blockquote.rs        # Blockquote
├── fenced_code_block.rs # Fenced Code Block (신규)
└── paragraph.rs         # Paragraph
```

## 테스트 현황
- **총 106개 테스트 통과**
- Paragraph: 5개
- Heading: 25개
- Thematic Break: 19개
- Blockquote: 30개
- Fenced Code Block: 27개 (신규)

## 알려진 제한사항

**코드 블록 안 빈 줄 문제:**
현재 파서는 `split("\n\n")`으로 블록을 나눕니다. 코드 블록 안에 빈 줄이 있으면 잘못 분리됩니다.
```markdown
```
code

more code
```
```
→ 이 입력이 두 개의 블록으로 잘못 나뉨

이 문제는 파서 구조 변경이 필요합니다 (블록 단위 분리 → 라인 단위 스캔).

## 다음 학습 시 시작점

1. **Indented Code Block**: 4칸 들여쓰기 코드 블록
2. **Setext Heading**: `===` 또는 `---` 스타일 제목
3. **List**: 순서 있는/없는 목록 (가장 복잡)
4. **파서 구조 개선**: 빈 줄 분리 문제 해결

## 커밋 히스토리

```
5f5dbb7 Fenced Code Block을 메인 파서에 통합
b7a3042 Fenced Code Block: 들여쓰기 처리 (0-3칸 허용, 내용에서 제거)
fa522f1 Fenced Code Block: 닫는 펜스 없음 처리 (EOF까지 코드)
b861c7a Fenced Code Block: 틸드 펜스(~~~) 및 펜스 길이 매칭 추가
19fc5d9 Fenced Code Block: info string 파싱 추가
eeb3f04 Fenced Code Block 기본 구현 (백틱 펜스)
```
