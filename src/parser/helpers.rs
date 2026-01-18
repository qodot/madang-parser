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

/// 앞뒤 빈 줄(빈 문자열) 제거 후 join
pub(crate) fn trim_blank_lines(lines: Vec<String>) -> String {
    let trimmed_lines: Vec<_> = lines
        .into_iter()
        .skip_while(|s| s.trim().is_empty())
        .collect();
    let mut trimmed_lines: Vec<_> = trimmed_lines
        .into_iter()
        .rev()
        .skip_while(|s| s.trim().is_empty())
        .collect();
    trimmed_lines.reverse();
    trimmed_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    // 빈 문자열
    #[case("", ' ', 0)]
    #[case("", '`', 0)]
    // 해당 문자가 없는 경우
    #[case("abc", ' ', 0)]
    #[case("abc", '`', 0)]
    // 공백 세기
    #[case(" ", ' ', 1)]
    #[case("  ", ' ', 2)]
    #[case("  abc", ' ', 2)]
    #[case("     ", ' ', 5)]           // 전체가 공백
    // 백틱 세기
    #[case("```", '`', 3)]
    #[case("`````code", '`', 5)]
    #[case("```rust", '`', 3)]
    // 틸드 세기
    #[case("~~~", '~', 3)]
    #[case("~~~~~", '~', 5)]
    // 탭은 공백으로 안 셈
    #[case("\tabc", ' ', 0)]
    #[case(" \tabc", ' ', 1)]          // 공백 1개 후 탭
    // 중간에 다른 문자
    #[case("  a  ", ' ', 2)]           // 앞 공백만 셈
    fn test_count_leading_char(#[case] input: &str, #[case] c: char, #[case] expected: usize) {
        assert_eq!(count_leading_char(input, c), expected);
    }

    #[rstest]
    // 기본 케이스
    #[case("  code", 2, "code")]
    #[case("    code", 4, "code")]
    // 부분 제거 (n보다 공백이 많음)
    #[case("    code", 2, "  code")]
    #[case("      code", 3, "   code")]
    // 공백 없는 경우
    #[case("code", 2, "code")]
    #[case("code", 0, "code")]
    // n이 공백보다 큰 경우
    #[case("  code", 5, "code")]
    #[case("  code", 100, "code")]
    // n이 0인 경우
    #[case("  code", 0, "  code")]
    // 빈 문자열
    #[case("", 2, "")]
    #[case("", 0, "")]
    // 전체가 공백
    #[case("    ", 2, "  ")]
    #[case("    ", 4, "")]
    #[case("    ", 10, "")]
    // 탭은 제거 안 됨 (공백만 제거)
    #[case("\tcode", 2, "\tcode")]
    #[case("  \tcode", 2, "\tcode")]   // 공백 2개 제거, 탭은 남음
    #[case("  \tcode", 3, "\tcode")]   // 공백 2개만 있으므로 2개만 제거
    fn test_remove_indent(#[case] input: &str, #[case] n: usize, #[case] expected: &str) {
        assert_eq!(remove_indent(input, n), expected);
    }

    #[rstest]
    // 공백만
    #[case("", 0)]
    #[case(" ", 1)]
    #[case("  ", 2)]
    #[case("   ", 3)]
    #[case("    ", 4)]
    // 탭 (1탭 = 4칸)
    #[case("\t", 4)]
    #[case("\t\t", 8)]
    // 공백 + 탭 혼합
    #[case(" \t", 5)]           // 1 + 4
    #[case("  \t", 6)]          // 2 + 4
    #[case("\t ", 5)]           // 4 + 1
    // 텍스트 포함
    #[case("code", 0)]
    #[case(" code", 1)]
    #[case("  code", 2)]
    #[case("\tcode", 4)]
    #[case("  \tcode", 6)]      // 2 + 4
    fn test_calculate_indent(#[case] input: &str, #[case] expected: usize) {
        assert_eq!(calculate_indent(input), expected);
    }

    #[rstest]
    // 빈 Vec
    #[case(vec![], "")]
    // 모두 빈 줄
    #[case(vec!["".to_string()], "")]
    #[case(vec!["".to_string(), "".to_string()], "")]
    #[case(vec!["  ".to_string()], "")]                    // 공백만 있는 줄도 빈 줄
    // 앞에만 빈 줄
    #[case(vec!["".to_string(), "code".to_string()], "code")]
    #[case(vec!["".to_string(), "".to_string(), "code".to_string()], "code")]
    #[case(vec!["  ".to_string(), "code".to_string()], "code")]  // 공백만 있는 앞줄
    // 뒤에만 빈 줄
    #[case(vec!["code".to_string(), "".to_string()], "code")]
    #[case(vec!["code".to_string(), "".to_string(), "".to_string()], "code")]
    #[case(vec!["code".to_string(), "  ".to_string()], "code")]  // 공백만 있는 뒷줄
    // 앞뒤 모두 빈 줄
    #[case(vec!["".to_string(), "code".to_string(), "".to_string()], "code")]
    #[case(vec!["".to_string(), "".to_string(), "code".to_string(), "".to_string(), "".to_string()], "code")]
    // 중간 빈 줄은 유지
    #[case(vec!["a".to_string(), "".to_string(), "b".to_string()], "a\n\nb")]
    #[case(vec!["".to_string(), "a".to_string(), "".to_string(), "b".to_string(), "".to_string()], "a\n\nb")]
    // 빈 줄 없음
    #[case(vec!["a".to_string()], "a")]
    #[case(vec!["a".to_string(), "b".to_string()], "a\nb")]
    #[case(vec!["a".to_string(), "b".to_string(), "c".to_string()], "a\nb\nc")]
    fn test_trim_blank_lines(#[case] input: Vec<String>, #[case] expected: &str) {
        assert_eq!(trim_blank_lines(input), expected);
    }
}
