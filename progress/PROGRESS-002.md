# 학습 진행 상황

## 마지막 학습일: 2026.1.14

## 이전 학습 요약 (PROGRESS-001)
- AST 개념, 러스트 enum, 소유권/참조, 모듈 시스템
- Node 타입: Document, Paragraph, Text
- parse() 함수: 단순 텍스트 파싱

## 오늘 완료된 내용

### 1. 여러 문단 파싱
- `\n\n` (빈 줄)로 문단 구분
- CommonMark 명세 확인: 빈 줄이 블록 구분자
- 이터레이터 체이닝: `split()` → `filter()` → `map()` → `collect()`

### 2. 엣지 케이스 처리
- **앞뒤 빈 줄**: `filter(|s| !s.is_empty())`로 빈 문자열 제거
- **연속 빈 줄**: `trim()`으로 앞뒤 공백/개행 제거
- CommonMark 명세에서 엣지 케이스 규칙 확인

### 3. ATX Heading 파싱
- Node에 `Heading { level: u8, children: Vec<Node> }` 추가
- CommonMark 명세 확인 후 구현:
  - 레벨 1~6 (`#` ~ `######`)
  - 7개 이상은 일반 Paragraph
  - `#` 뒤 공백 필수

### 4. 코드 리팩토링
- **`impl Node` 블록 추가**: enum에 메서드 정의
- **`children()` 메서드**: Container 노드의 children 반환
- **`as_text()` 메서드**: Text 노드의 문자열 반환
- **`level()` 메서드**: Heading 노드의 레벨 반환
- 테스트 코드 50% 이상 감소 (224줄 삭제, 60줄 추가)

## 오늘 배운 러스트 개념

### 이터레이터 메서드
```rust
input.split("\n\n")        // Iterator<&str>
    .filter(|s| !s.is_empty())  // 조건 필터링
    .map(|block| ...)           // 변환
    .collect()                  // Vec으로 수집
```

### impl 블록
```rust
impl Node {
    pub fn children(&self) -> &Vec<Node> { ... }
}
```
- enum에 메서드 추가
- `&self`: 불변 참조로 자기 자신 빌림

### 패턴 매칭
- `..`: 나머지 필드 무시 (`Node::Heading { children, .. }`)
- `*c`: 역참조 (참조에서 값 추출)

### take_while과 참조
- `take_while(|c| ...)`: `&char` 전달 (빌려서 검사)
- 검사 후 값을 다음 단계로 넘겨야 하므로 참조 사용

## 현재 코드 구조

```
src/
├── lib.rs      # 모듈 선언 + re-export
├── node.rs     # Node enum + impl + 테스트
└── parser.rs   # parse() 함수 + 테스트
```

### Node 타입
```rust
pub enum Node {
    Document { children: Vec<Node> },
    Heading { level: u8, children: Vec<Node> },
    Paragraph { children: Vec<Node> },
    Text(String),
}

impl Node {
    pub fn children(&self) -> &Vec<Node>;
    pub fn as_text(&self) -> &str;
    pub fn level(&self) -> u8;
}
```

## 다음 학습 시 시작점

다음 중 하나를 선택하여 진행:

1. **Thematic Break**: `---`, `***`, `___` 구분선
2. **Code Block**: ``` 코드 블록
3. **닫는 # 처리**: `# title #` 형식
4. **Block/Inline 타입 분리**: 타입 안전성 향상

## 커밋 히스토리

```
12b7199 Node 메서드 추가 및 테스트 리팩토링
092aaad Heading 엣지 케이스 테스트 추가 (h6, 7개 #)
8b2ac97 Heading: # 뒤 공백 필수 규칙 적용
c7dd85b ATX Heading 파싱 구현
9f26eda Node에 Heading variant 추가
383b717 엣지 케이스 처리: filter + trim 추가
746b077 스펙 확인 후 구현하는 지침 추가
9e132a2 여러 문단 파싱 구현
48ca575 학습 진행 저장 방식 변경: 시퀀스 기반으로
```
