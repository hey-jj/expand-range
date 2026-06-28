//! Compile a numeric interval into a compact regex source string.
//!
//! This generates a regex fragment that matches every integer in `[min, max]`
//! and nothing else. It splits the interval into digit ranges, builds character
//! classes and quantifiers, and handles negative numbers and zero padding.
//!
//! Inputs arrive as strings so leading zeros survive and drive padding output.

/// Settings that change the emitted pattern.
pub(crate) struct RegexOptions {
    /// Allow optional leading zeros (`0?`, `0{0,2}`) when padded.
    pub relax_zeros: bool,
    /// Use `\d` instead of `[0-9]`.
    pub shorthand: bool,
    /// Wrap the whole pattern in a capturing group.
    pub capture: bool,
    /// Auto-wrap multi-pattern output in a non-capturing group.
    ///
    /// The step-one path passes this through from the user. The stepped path
    /// passes false and does its own wrapping.
    pub wrap: bool,
}

/// State carried through the split: whether the interval is padded and the
/// width to pad to.
struct State {
    is_padded: bool,
    max_len: usize,
}

/// One generated sub-pattern.
#[derive(Clone)]
struct Token {
    pattern: String,
    /// Quantifier counts. `[start]` or `[start, stop]`.
    count: Vec<i64>,
    string: String,
}

/// True when a numeric string has a leading zero followed by a digit.
///
/// Matches `/^-?(0+)\d/`: an optional sign, one or more zeros, then any digit.
/// `"05"`, `"00"`, and `"-00"` are padded. `"0"` and `"5"` are not. A lone zero
/// fails because it has no digit after the zero run.
fn has_padding(s: &str) -> bool {
    let body = s.strip_prefix('-').unwrap_or(s);
    let mut chars = body.chars();
    if chars.next() != Some('0') {
        return false;
    }
    // The regex is not anchored at the end. A leading zero plus one more digit
    // is enough. Anything after that digit does not matter.
    matches!(chars.next(), Some(c) if c.is_ascii_digit())
}

/// The largest value with `len` trailing nines that shares a prefix with `min`.
///
/// Saturates to `i64::MAX` when the constructed number overflows, which keeps the
/// `split_to_ranges` loop bounded instead of panicking on wide inputs.
fn count_nines(min: i64, len: usize) -> i64 {
    let s = min.to_string();
    let keep = s.len().saturating_sub(len);
    let prefix = &s[..keep];
    let nines: String = std::iter::repeat_n('9', len).collect();
    format!("{prefix}{nines}").parse().unwrap_or(i64::MAX)
}

/// Round `integer` down to a multiple of `10^zeros`.
///
/// Saturates the power of ten so a large `zeros` cannot overflow.
fn count_zeros(integer: i64, zeros: u32) -> i64 {
    let modulus = 10_i64.checked_pow(zeros).unwrap_or(i64::MAX);
    integer - integer.rem_euclid(modulus)
}

/// Build the quantifier text for a count pair.
fn to_quantifier(digits: &[i64]) -> String {
    let start = digits.first().copied().unwrap_or(0);
    let stop = digits.get(1).copied();
    match stop {
        Some(stop) if stop != 0 => format!("{{{start},{stop}}}"),
        _ if start > 1 => format!("{{{start}}}"),
        _ => String::new(),
    }
}

/// Character class for a digit range, collapsing adjacent pairs.
fn to_character_class(a: u8, b: u8) -> String {
    let dash = if b - a == 1 { "" } else { "-" };
    format!("[{}{}{}]", a as char, dash, b as char)
}

/// Padding prefix for a value given the target width.
fn pad_zeros(value: i64, st: &State, relax: bool) -> String {
    if !st.is_padded {
        return value.to_string();
    }
    let len = value.abs().to_string().len();
    let diff = st.max_len.abs_diff(len);
    match diff {
        0 => String::new(),
        1 => {
            if relax {
                "0?".to_string()
            } else {
                "0".to_string()
            }
        }
        2 => {
            if relax {
                "0{0,2}".to_string()
            } else {
                "00".to_string()
            }
        }
        _ => {
            if relax {
                format!("0{{0,{diff}}}")
            } else {
                format!("0{{{diff}}}")
            }
        }
    }
}

/// Split `[min, max]` at the boundaries where digit patterns change.
fn split_to_ranges(min: i64, max: i64) -> Vec<i64> {
    let mut stops: Vec<i64> = vec![max];
    // An i64 has at most 19 decimal digits. Past that width no new boundary
    // appears, and the cap stops the loops from spinning when count_nines or
    // count_zeros saturates at the i64 limit.
    let max_width = 19usize;
    let mut nines = 1usize;
    let mut stop = count_nines(min, nines);
    while nines <= max_width && min <= stop && stop <= max {
        if !stops.contains(&stop) {
            stops.push(stop);
        }
        nines += 1;
        stop = count_nines(min, nines);
    }
    let mut zeros = 1u32;
    let ceiling = max.saturating_add(1);
    let mut stop = count_zeros(ceiling, zeros) - 1;
    while (zeros as usize) <= max_width && min < stop && stop <= max {
        if !stops.contains(&stop) {
            stops.push(stop);
        }
        zeros += 1;
        stop = count_zeros(ceiling, zeros) - 1;
    }
    stops.sort_unstable();
    stops
}

/// Build the pattern for a single `[start, stop]` digit range.
fn range_to_pattern(start: &str, stop: &str, shorthand: bool) -> Token {
    if start == stop {
        return Token {
            pattern: start.to_string(),
            count: vec![],
            string: String::new(),
        };
    }
    let sb = start.as_bytes();
    let tb = stop.as_bytes();
    let digits = sb.len();
    let mut pattern = String::new();
    let mut count = 0i64;
    for i in 0..digits {
        let start_digit = sb[i];
        let stop_digit = tb[i];
        if start_digit == stop_digit {
            pattern.push(start_digit as char);
        } else if start_digit != b'0' || stop_digit != b'9' {
            pattern.push_str(&to_character_class(start_digit, stop_digit));
        } else {
            count += 1;
        }
    }
    if count > 0 {
        pattern.push_str(if shorthand { "\\d" } else { "[0-9]" });
    }
    Token {
        pattern,
        count: vec![count],
        string: String::new(),
    }
}

/// Build tokens spanning `[min, max]`, merging repeats and applying padding.
fn split_to_patterns(min: i64, max: i64, st: &State, relax: bool, shorthand: bool) -> Vec<Token> {
    let ranges = split_to_ranges(min, max);
    let mut tokens: Vec<Token> = vec![];
    let mut start = min;
    for max in ranges {
        let obj = range_to_pattern(&start.to_string(), &max.to_string(), shorthand);
        if !st.is_padded {
            if let Some(prev) = tokens.last_mut() {
                if prev.pattern == obj.pattern {
                    if prev.count.len() > 1 {
                        prev.count.pop();
                    }
                    prev.count.push(obj.count[0]);
                    prev.string = format!("{}{}", prev.pattern, to_quantifier(&prev.count));
                    start = max.saturating_add(1);
                    continue;
                }
            }
        }
        let zeros = if st.is_padded {
            pad_zeros(max, st, relax)
        } else {
            String::new()
        };
        let string = format!("{}{}{}", zeros, obj.pattern, to_quantifier(&obj.count));
        tokens.push(Token {
            pattern: obj.pattern,
            count: obj.count,
            string,
        });
        start = max.saturating_add(1);
    }
    tokens
}

/// Whether any token in `comparison` shares the `string` of `ele`.
fn contains(arr: &[Token], val: &str) -> bool {
    arr.iter().any(|t| t.string == val)
}

/// Keep tokens that do or do not intersect `comparison`, prefixing each.
fn filter_patterns(
    arr: &[Token],
    comparison: &[Token],
    prefix: &str,
    intersection: bool,
) -> Vec<String> {
    let mut result = vec![];
    for ele in arr {
        let present = contains(comparison, &ele.string);
        if (!intersection && !present) || (intersection && present) {
            result.push(format!("{}{}", prefix, ele.string));
        }
    }
    result
}

/// Combine negative-only, shared, and positive-only sub-patterns.
fn collate_patterns(neg: &[Token], pos: &[Token]) -> String {
    let only_negative = filter_patterns(neg, pos, "-", false);
    let only_positive = filter_patterns(pos, neg, "", false);
    let intersected = filter_patterns(neg, pos, "-?", true);
    let mut subpatterns = only_negative;
    subpatterns.extend(intersected);
    subpatterns.extend(only_positive);
    subpatterns.join("|")
}

/// Compile `[min, max]` into a regex source string.
///
/// `min` and `max` are textual so leading zeros drive padding. The interval is
/// inclusive. The result matches every integer in range and nothing else.
///
/// Returns `None` when a bound does not fit in `i64`. The caller routes that to
/// the same empty-list or strict-error path it uses for other invalid input, so
/// no input can make this panic.
pub(crate) fn to_regex_range(min: &str, max: &str, opts: &RegexOptions) -> Option<String> {
    if min == max {
        return Some(min.to_string());
    }

    let min_n: i64 = min.parse().ok()?;
    let max_n: i64 = max.parse().ok()?;

    let mut a = min_n.min(max_n);
    let b = min_n.max(max_n);

    if a.abs_diff(b) == 1 {
        let result = format!("{min}|{max}");
        if opts.capture {
            return Some(format!("({result})"));
        }
        if !opts.wrap {
            return Some(result);
        }
        return Some(format!("(?:{result})"));
    }

    let is_padded = has_padding(min) || has_padding(max);
    let st = State {
        is_padded,
        // Mirrors String(state.max).length: the textual length of the max arg.
        max_len: if is_padded { max.len() } else { 0 },
    };

    let mut positives: Vec<Token> = vec![];
    let mut negatives: Vec<Token> = vec![];

    if a < 0 {
        // `i64::MIN.abs()` has no positive counterpart, so use the unsigned
        // magnitude and saturate. Bounds this large never match real input, and
        // the split below stays correct for every representable value.
        let new_min = if b < 0 {
            b.unsigned_abs().min(i64::MAX as u64) as i64
        } else {
            1
        };
        let a_abs = a.unsigned_abs().min(i64::MAX as u64) as i64;
        negatives = split_to_patterns(new_min, a_abs, &st, opts.relax_zeros, opts.shorthand);
        a = 0;
    }

    if b >= 0 {
        positives = split_to_patterns(a, b, &st, opts.relax_zeros, opts.shorthand);
    }

    let mut result = collate_patterns(&negatives, &positives);

    if opts.capture {
        result = format!("({result})");
    } else if opts.wrap && (positives.len() + negatives.len()) > 1 {
        result = format!("(?:{result})");
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_padding_matches_leading_zero_then_digit() {
        // Truth table for /^-?(0+)\d/.
        assert!(!has_padding("0"));
        assert!(has_padding("00"));
        assert!(has_padding("000"));
        assert!(has_padding("01"));
        assert!(has_padding("05"));
        assert!(has_padding("-05"));
        assert!(!has_padding("-0"));
        assert!(has_padding("-00"));
        assert!(!has_padding("0a"));
        assert!(has_padding("00a"));
        assert!(!has_padding("10"));
        assert!(has_padding("020"));
        assert!(!has_padding("5"));
    }

    #[test]
    fn wide_bounds_return_none_instead_of_panicking() {
        let opts = RegexOptions {
            relax_zeros: true,
            shorthand: false,
            capture: false,
            wrap: false,
        };
        // A 19-digit bound exceeds i64::MAX.
        assert_eq!(to_regex_range("1", "9999999999999999999", &opts), None);
    }

    #[test]
    fn i64_extremes_do_not_panic() {
        let opts = RegexOptions {
            relax_zeros: true,
            shorthand: false,
            capture: false,
            wrap: false,
        };
        // 18 nines fits i64 but count_nines builds a wider all-nines string.
        assert!(to_regex_range("1", "999999999999999999", &opts).is_some());
        // i64::MIN has no positive counterpart for abs().
        let min = i64::MIN.to_string();
        assert!(to_regex_range(&min, "0", &opts).is_some());
        // i64::MAX as the upper bound exercises the max+1 path.
        let max = i64::MAX.to_string();
        assert!(to_regex_range("1", &max, &opts).is_some());
    }
}
