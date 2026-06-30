//! Pin the to_regex branches that the other suites do not cover:
//! single-step capture and wrap, the adjacent-pair shortcut, the wide padded
//! vector, and step resolution for empty and coerced forms.

mod common;

use common::{exact, regex_eq, s};
use expand_range::{expand, FillResult, Options, Step, Value};

/// Run a numeric to_regex expand with the given option tweaks.
fn rx(a: i64, b: i64, set: impl FnOnce(&mut Options)) -> FillResult {
    let mut o = Options::new();
    o.to_regex = true;
    set(&mut o);
    expand(Value::from(a), Some(Value::from(b)), Step::None, o)
}

/// Run a string-bound to_regex expand with the given option tweaks.
fn rxs(a: &str, b: &str, set: impl FnOnce(&mut Options)) -> FillResult {
    let mut o = Options::new();
    o.to_regex = true;
    set(&mut o);
    expand(Value::from(a), Some(Value::from(b)), Step::None, o)
}

#[test]
fn single_step_capture_wraps_the_alternation() {
    regex_eq(rx(2, 8, |o| o.capture = true), "([2-8])");
    regex_eq(rx(2, 10, |o| o.capture = true), "([2-9]|10)");
    regex_eq(rx(2, 100, |o| o.capture = true), "([2-9]|[1-9][0-9]|100)");
    regex_eq(rx(-10, 10, |o| o.capture = true), "(-[1-9]|-?10|[0-9])");
}

#[test]
fn single_step_wrap_only_groups_multiple_subpatterns() {
    // One sub-pattern stays unwrapped even with wrap set.
    regex_eq(rx(2, 8, |o| o.wrap = true), "[2-8]");
    // More than one sub-pattern gets a non-capturing group.
    regex_eq(rx(2, 10, |o| o.wrap = true), "(?:[2-9]|10)");
}

#[test]
fn all_zeros_lower_bound_keeps_the_padding_group() {
    regex_eq(rxs("00", "10", |_| {}), "0?[0-9]|10");
    regex_eq(rxs("00", "20", |_| {}), "0?[0-9]|1[0-9]|20");
    regex_eq(rxs("000", "100", |_| {}), "0{0,2}[0-9]|0?[1-9][0-9]|100");
    regex_eq(rxs("00", "99", |_| {}), "0?[0-9]|[1-9][0-9]");

    // The pattern must accept every padded member 00..09.
    let source = match rxs("00", "10", |_| {}) {
        FillResult::Regex(r) => r,
        FillResult::List(_) => panic!("expected regex"),
    };
    let re = regex::Regex::new(&format!("^({source})$")).expect("valid regex");
    for n in 0..=9 {
        let padded = format!("{n:02}");
        assert!(re.is_match(&padded), "{padded} should match {source}");
    }
}

#[test]
fn adjacent_pair_emits_min_or_max() {
    regex_eq(rx(9, 10, |_| {}), "9|10");
    regex_eq(rx(9, 10, |o| o.wrap = true), "(?:9|10)");
    regex_eq(rx(9, 10, |o| o.capture = true), "(9|10)");
    // The padded variant keeps the literal 09 from the textual bounds.
    regex_eq(rxs("09", "10", |_| {}), "09|10");
}

#[test]
fn wide_padded_vector_uses_relaxed_quantifiers() {
    regex_eq(
        rxs("000001", "100000", |_| {}),
        "0{0,5}[1-9]|0{0,4}[1-9][0-9]|0{0,3}[1-9][0-9]{2}|0{0,2}[1-9][0-9]{3}|0?[1-9][0-9]{4}|100000",
    );
    // The strict-zeros variant pins the fixed-width form.
    regex_eq(
        rxs("000001", "100000", |o| o.strict_zeros = true),
        "0{5}[1-9]|0{4}[1-9][0-9]|0{3}[1-9][0-9]{2}|00[1-9][0-9]{3}|0[1-9][0-9]{4}|100000",
    );
}

#[test]
fn letter_regex_escapes_metacharacters() {
    // A bare "." would match any character, so an equal punctuation bound escapes.
    regex_eq(rxs(".", ".", |_| {}), "\\.");

    // Stepped punctuation members escape, so the join stays a literal
    // alternation instead of a character class that matches "|".
    let mut o = Options::new();
    o.to_regex = true;
    let source = match expand(Value::from("["), Some(Value::from("]")), Step::from(2), o) {
        FillResult::Regex(r) => r,
        FillResult::List(_) => panic!("expected regex"),
    };
    assert_eq!(source, "\\[|\\]");
    let re = regex::Regex::new(&format!("^({source})$")).expect("valid regex");
    assert!(re.is_match("["));
    assert!(re.is_match("]"));
    assert!(!re.is_match("|"));
}

#[test]
fn empty_string_step_falls_through_to_one() {
    exact(
        expand(
            Value::from("1"),
            Some(Value::from("5")),
            Step::from(""),
            Options::new(),
        ),
        &[s("1"), s("2"), s("3"), s("4"), s("5")],
    );
}

#[test]
fn bound_coercion_at_the_number_letter_boundary() {
    // "1." coerces to the integer 1, so the range is a single value.
    exact(
        expand(
            Value::from("1"),
            Some(Value::from("1.")),
            Step::None,
            Options::new(),
        ),
        &[s("1")],
    );
    // A leading plus coerces to a number.
    exact(
        expand(
            Value::from("1"),
            Some(Value::from("+5")),
            Step::None,
            Options::new(),
        ),
        &[s("1"), s("2"), s("3"), s("4"), s("5")],
    );
    exact(
        expand(
            Value::from("+2"),
            Some(Value::from("+5")),
            Step::None,
            Options::new(),
        ),
        &[s("2"), s("3"), s("4"), s("5")],
    );
    // ".5" is not an integer, so the range is empty.
    exact(
        expand(
            Value::from("1"),
            Some(Value::from(".5")),
            Step::None,
            Options::new(),
        ),
        &[],
    );
    // A step "1." coerces to the integer 1.
    exact(
        expand(
            Value::from("1"),
            Some(Value::from("5")),
            Step::from("1."),
            Options::new(),
        ),
        &[s("1"), s("2"), s("3"), s("4"), s("5")],
    );
}
