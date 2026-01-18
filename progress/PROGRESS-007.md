# 학습 진행 상황 기록 (PROGRESS-007)

## 날짜
2026-01-18

## 이번 세션 요약
Setext Heading 파싱 구현 및 테스트 조직 개선

## 구현한 내용

### 1. Setext Heading 파싱
- `=` 밑줄 → 레벨 1
- `-` 밑줄 → 레벨 2

**파일 구조**:
- `heading_setext.rs`: `try_start` 함수 (밑줄 검사)
- `context.rs`: 타입 정의
  - `SetextLevel` (Level1, Level2)
  - `HeadingSetextStart`
  - `HeadingSetextStartReason::Started`
  - `HeadingSetextNotStartReason` (IndentedCodeBlock, Empty, NotUnderlineChar, MixedChars)
- `mod.rs`: 통합 (process_line_in_paragraph에서 Thematic Break보다 먼저 확인)
- `node.rs`: `is_heading()` 메서드 추가

### 2. 테스트 조직 개선
각 기능 모듈에 관련 테스트 배치:
- `test_setext_heading` → `heading_setext.rs`로 이동
- `code_block_with_blank_line` → `fenced_code_block.rs`로 이동
- `mod.rs`에는 전역 `parse` 함수 기본 테스트만 유지

### 3. 엣지 케이스 테스트
- 밑줄 들여쓰기 (1-3칸 허용, 4칸 이상 Setext 아님)
- 제목 텍스트 들여쓰기 (1-3칸 허용)
- 빈 줄 후 밑줄 → Setext 아님
- 밑줄만 단독 (`===` → Paragraph, `---` → Thematic Break)
- 밑줄 뒤 비공백 문자 → Setext 아님

## 학습한 내용

### Setext Heading 명세 (CommonMark)
- 밑줄: `=` 또는 `-` 문자만, 내부 공백 불가, 후행 공백 허용
- 들여쓰기: 제목 텍스트와 밑줄 모두 0-3칸 허용
- 우선순위: Paragraph 컨텍스트에서 `---`는 Thematic Break가 아닌 Setext 밑줄로 해석

### Paragraph 처리 규칙
- **빈 줄 없이 연속**: 다음 줄이 블록 요소가 아니면 continuation (하나의 Paragraph)
- **빈 줄로 구분**: 별도의 블록들로 파싱
- Indented Code Block은 Paragraph를 인터럽트할 수 없음

### 테스트 경로 규칙
- `tests` 모듈 내에서 `super::parse`는 부모 모듈을 가리킴
- mod.rs의 parse 함수에 접근하려면 `crate::parse` 사용

## 테스트 현황
- 이전: 333개
- 현재: 347개 (+14)
- 모두 통과

## 커밋 내역
- `24a0af7 feat: Setext Heading 파싱 구현 (=, - 밑줄 스타일)`
- `617f18f test: Setext Heading 엣지 케이스 테스트 추가`

## 다음 학습 시 선택지
1. **Indented Code Block** - 4칸 들여쓰기 코드 블록
2. **중첩 리스트** - 리스트 안의 리스트 (재귀 파싱)
3. **인라인 파싱** - 강조, 링크, 코드 스팬 등

## 현재 구현 완료된 블록 요소
- [x] ATX Heading (`#` 스타일)
- [x] Setext Heading (`=`, `-` 밑줄 스타일)
- [x] Paragraph
- [x] Thematic Break (`---`, `***`, `___`)
- [x] Fenced Code Block (` ``` `, `~~~`)
- [x] Blockquote (`>`)
- [x] List (bullet, ordered) - 다중 라인, 빈 줄 지원
- [ ] Indented Code Block
- [ ] 중첩 리스트
- [ ] 인라인 파싱
