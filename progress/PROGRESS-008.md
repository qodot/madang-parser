# 학습 진행 상황 (PROGRESS-008)

## 날짜
2026-01-18

## 이번 세션 학습 내용

### Indented Code Block 파싱 구현

CommonMark 명세에 따라 Indented Code Block 파싱을 구현했다.

#### 핵심 규칙 (CommonMark 명세)
1. **4칸 이상 들여쓰기**: 4칸 스페이스로 시작하는 줄이 코드 블록
2. **Paragraph 인터럽트 불가**: 빈 줄 없이 Paragraph 뒤에 4칸 들여쓰기가 오면 Paragraph의 일부로 처리
3. **List 우선**: List 컨텍스트에서는 List가 우선권을 가짐
4. **내부 빈 줄 보존**: 코드 블록 내부의 빈 줄은 그대로 유지
5. **앞뒤 빈 줄 제거**: 코드 블록 시작/끝의 빈 줄은 제거

#### 구현된 파일들

1. **`src/parser/indented_code_block.rs`** (신규)
   - `try_start()`: 줄이 Indented Code Block 시작인지 확인
   - Result 타입으로 성공/실패 사유 반환
   - 통합 테스트: CommonMark Examples 107, 110-118

2. **`src/parser/context.rs`** (수정)
   - `IndentedCodeBlockStart`: 시작 정보 구조체
   - `IndentedCodeBlockStartReason`: 시작 성공 사유
   - `IndentedCodeBlockNotStartReason`: 시작 실패 사유 (Empty, InsufficientIndent)
   - `ParsingContext::IndentedCodeBlock`: 파싱 상태 추가

3. **`src/parser/mod.rs`** (수정)
   - `process_line_in_indented_code_block()`: 상태 처리 함수
   - `trim_blank_lines()`: 앞뒤 빈 줄 제거 헬퍼

#### 주요 학습 포인트

1. **빈 줄 처리 전략 (pending_blank_count)**
   - 빈 줄이 오면 바로 추가하지 않고 `pending_blank_count`로 대기
   - 다음 코드 줄이 오면 대기 중인 빈 줄들을 추가
   - 코드 블록이 끝나면 대기 중인 빈 줄은 버림

2. **들여쓰기 우선 확인**
   - `try_start`에서 들여쓰기를 먼저 확인하고 빈 줄 여부를 나중에 확인
   - 4칸 이상 들여쓰기된 빈 줄(예: `"      "`)도 코드의 일부로 처리

3. **match 표현식으로 명확한 분기**
   - `try_start`의 Result를 match로 처리하여 모든 케이스를 명시적으로 다룸
   - `Ok(Started)`, `Err(Empty)`, `Err(InsufficientIndent)` 각각 다른 처리

#### 테스트된 CommonMark 예제

| Example | 설명 |
|---------|------|
| 107 | 기본 코드 블록 |
| 110 | HTML/마크다운이 그대로 코드로 처리 |
| 111 | 빈 줄로 분리된 청크들이 하나의 블록 |
| 112 | 들여쓰기된 빈 줄 유지 |
| 113 | Paragraph 인터럽트 불가 |
| 114 | 코드 블록 후 4칸 미만 줄은 새 Paragraph |
| 116 | 8칸 들여쓰기 (4칸 제거 후 4칸 유지) |
| 117 | 앞뒤 빈 줄 제거 |
| 118 | 후행 공백 유지 |

## 현재 상태

### 테스트 현황
- **361개 테스트 통과** (이전: 347개, +14개)

### 구현 완료된 블록 요소
1. ✅ Thematic Break (`---`, `***`, `___`)
2. ✅ ATX Heading (`# ~ ######`)
3. ✅ Setext Heading (`===`, `---` 밑줄)
4. ✅ Fenced Code Block (`` ``` ``, `~~~`)
5. ✅ Indented Code Block (4칸 들여쓰기) ← **이번 세션**
6. ✅ Paragraph
7. ✅ Blockquote (`>`)
8. ✅ List (bullet, ordered)

### 다음 학습 옵션
1. **중첩 리스트 개선**: 현재 단순 중첩만 지원, 더 복잡한 케이스 처리
2. **Inline 파싱 시작**: 강조(`*`, `_`), 링크, 이미지 등
3. **HTML 블록**: CommonMark의 HTML 블록 지원
4. **Link Reference Definition**: `[label]: url` 형식
