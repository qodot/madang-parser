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
}
