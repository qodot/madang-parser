# 학습 진행 상황

## 마지막 학습일: 2026.1.17

## 이전 학습 요약 (PROGRESS-002)
- 여러 문단 파싱 (`\n\n`으로 구분)
- ATX Heading 파싱 (레벨 1~6, 닫는 # 처리)
- Node 메서드: `children()`, `as_text()`, `level()`

## 오늘 완료된 내용

### 1. Thematic Break 구현
- `***`, `---`, `___` 구분선 파싱
- 마커 3개 이상, 같은 문자만, 공백/탭 허용
- 들여쓰기 3칸 이하만 유효

### 2. Blockquote 구현
- 기본: `>` 마커로 시작하는 인용문
- 중첩: `> > nested` 재귀 파싱
- 다중줄: 각 줄에서 `>` 마커 제거
- Lazy continuation: `>` 없는 줄도 포함
- 복수 단락: `>\n>` (빈 `>` 줄)로 분리
- 내부 블록 요소: Heading, ThematicBreak 지원

### 3. 파서 모듈 구조 분리
```
src/parser/
├── mod.rs              # 디스패처 (parse, parse_block)
├── heading.rs          # ATX Heading
├── thematic_break.rs   # Thematic Break
├── blockquote.rs       # Blockquote
└── paragraph.rs        # Paragraph (fallback)
```

### 4. 테스트 리팩토링
- 모든 테스트를 rstest parametrized 테스트로 통합
- `#[case]`로 케이스 추가, 중복 코드 제거
- `Option<T>`로 "있음/없음" 케이스 통합

## 오늘 배운 러스트 개념

### 디렉토리 모듈
```rust
// parser.rs → parser/mod.rs 변환
// lib.rs에서 `mod parser;`는 그대로 유지
```

### 함수를 파라미터로 전달 (클로저/제네릭)
```rust
pub fn parse<F>(trimmed: &str, indent: usize, parse_block: F) -> Option<Node>
where
    F: Fn(&str) -> Node,
```

### Iterator 체이닝
```rust
text.lines()
    .map(|line| /* 처리 */)
    .collect::<Vec<_>>()
    .join("\n")
```

### Option 체이닝 (or_else)
```rust
thematic_break::parse(trimmed, indent)
    .or_else(|| blockquote::parse(...))
    .or_else(|| heading::parse(...))
    .unwrap_or_else(|| paragraph::parse(trimmed))
```

## Rust 컨벤션
- 공개(pub) 함수를 비공개 함수보다 위에 배치
- 테스트는 파일 맨 아래 `#[cfg(test)] mod tests`

## 현재 코드 구조

### Node 타입
```rust
pub enum Node {
    Document { children: Vec<Node> },
    Heading { level: u8, children: Vec<Node> },
    Paragraph { children: Vec<Node> },
    Blockquote { children: Vec<Node> },
    ThematicBreak,
    Text(String),
}
```

### 테스트 현황
- 총 79개 테스트 통과
- Paragraph: 5개
- Heading: 25개
- Thematic Break: 19개
- Blockquote: 30개

## 다음 학습 시 시작점

다음 중 하나를 선택하여 진행:

1. **Indented Code Block**: 4칸 들여쓰기 코드 블록
2. **Fenced Code Block**: ``` 또는 ~~~ 코드 블록
3. **Setext Heading**: === 또는 --- 스타일 제목
4. **List**: 순서 있는/없는 목록

## 커밋 히스토리

```
7b8a458 Blockquote 내부 블록 테스트를 rstest로 통합
54c4764 Blockquote 내 블록 요소(Heading, ThematicBreak) 테스트 추가
0162621 Blockquote 내 복수 단락 지원 (빈 > 줄로 분리)
f3d8f9c Blockquote lazy continuation 테스트 추가
4568e9f Blockquote 다중줄 지원 (각 줄에서 > 마커 제거)
b7f9ec5 Rust 컨벤션 적용: public 함수를 private 함수보다 위에 배치
77f12fb 파서 모듈 구조 분리 (스펙별 파일)
d189c63 테스트 assert 메시지 한글화
d38d7c8 Blockquote 테스트 통합 (단순 + 중첩)
f260218 Blockquote 파싱 구현 (중첩 지원)
48029e4 Thematic Break 파싱 구현
```
