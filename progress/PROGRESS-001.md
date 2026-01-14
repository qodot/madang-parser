# 학습 진행 상황

## 마지막 학습일: 2025.1.14

## 완료된 내용

### 1. AST (Abstract Syntax Tree) 개념
- AST는 파서가 텍스트를 읽고 만들어내는 **구조화된 데이터**
- "텍스트 → 파서 → AST → 렌더러 → HTML" 흐름 이해
- **추상**의 의미: 문법 기호(`#`, `**`)는 버리고 의미만 남김

### 2. 러스트 enum
- **합 타입(sum type)**: "A이거나 B이거나 C" 표현
- **세 가지 variant 형태**:
  - Unit: `Empty`
  - Tuple: `Text(String)` - 이름 없이 순서로
  - Struct: `Document { children: Vec<Node> }` - 이름 있는 필드
- **Exhaustive matching**: 모든 variant를 처리해야 함

### 3. 러스트 소유권/참조
- `&` = 빌려서 보기만 (소유권 이동 없음)
- `&str` vs `String`: 빌린 것 vs 소유하는 것
- `.to_string()`: `&str` → `String` 변환

### 4. 러스트 모듈 시스템
- 파일 = 모듈 (`node.rs` = `node` 모듈)
- `mod X;` → 파일을 모듈로 불러옴
- `use crate::X;` → 같은 crate 내 다른 모듈 참조
- `pub use X::Y;` → 외부에 re-export

### 5. TDD 방식
- 테스트 먼저 작성 → 실패 확인 → 최소 구현

## 현재 코드 구조

```
src/
├── lib.rs      # 모듈 선언 + re-export
├── node.rs     # Node enum + 테스트
└── parser.rs   # parse() 함수 + 테스트
```

### Node 타입
```rust
pub enum Node {
    Document { children: Vec<Node> },
    Paragraph { children: Vec<Node> },
    Text(String),
}
```

### parse() 함수
- 빈 문자열 → 빈 Document
- 텍스트 → Document > Paragraph > Text

## 다음 학습 시 시작점

다음 중 하나를 선택하여 진행:

1. **여러 문단 파싱**: 빈 줄로 구분된 여러 Paragraph
2. **Heading 파싱**: `# 제목` 형식의 ATX Heading
3. **코드 리팩토링**: 중복되는 match 패턴 개선

## 커밋 히스토리

```
a18ff8e TDD 방식 및 학습 종료 시 요약 저장 지침 추가
8a1a87a 단순 텍스트 파싱 구현
2a5927e node, parser 모듈 분리 및 테스트 분리
cefa0ea 빈 Document 반환하는 parse() 함수 추가
083588c AST Paragraph 노드 추가
c8909bc 작업 방식에 AskUserQuestion 사용 지침 추가
f03ff8f AST Text 노드 추가
361aa69 AST Document 노드 정의
a54809d 개발 지침 추가
7d94d3c 초기 프로젝트 설정
```
