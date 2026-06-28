//! Edge cases beyond the core suite, pinned to the documented behavior.

mod common;

use common::{exact, n, regex_eq, s};
use expand_range::{expand, expand_checked, FillError, FillResult, Options, Step, Value};

/// Strict options.
fn strict() -> Options {
    let mut o = Options::new();
    o.strict_ranges = true;
    o
}

#[test]
fn empty_start_non_strict_is_empty() {
    exact(
        expand(
            Value::from(""),
            Some(Value::from("5")),
            Step::None,
            Options::new(),
        ),
        &[],
    );
}

#[test]
fn empty_start_strict_errors() {
    let e = expand_checked(
        Value::from(""),
        Some(Value::from("5")),
        Step::None,
        strict(),
    )
    .expect_err("expected an error");
    assert!(matches!(e, FillError::Range(_)));
}

#[test]
fn single_invalid_start_is_empty() {
    exact(
        expand(Value::from(""), None, Step::None, Options::new()),
        &[],
    );
}

#[test]
fn single_valid_value_returns_itself() {
    exact(
        expand(Value::from(5), None, Step::None, Options::new()),
        &[n(5.0)],
    );
    exact(
        expand(Value::from("1"), None, Step::None, Options::new()),
        &[s("1")],
    );
    exact(
        expand(Value::from("a"), None, Step::None, Options::new()),
        &[s("a")],
    );
    exact(
        expand(Value::from(0), None, Step::None, Options::new()),
        &[n(0.0)],
    );
}

#[test]
fn float_step_is_invalid() {
    // A non-integer step fails the integer check, like a junk step.
    exact(
        expand(
            Value::from(1),
            Some(Value::from(10)),
            Step::from("1.5"),
            Options::new(),
        ),
        &[],
    );
    let e = expand_checked(
        Value::from(1),
        Some(Value::from(10)),
        Step::from("1.5"),
        strict(),
    )
    .expect_err("expected an error");
    assert_eq!(
        e,
        FillError::Step("Expected step \"1.5\" to be a number".to_string())
    );
}

#[test]
fn letter_step_over_span_yields_single() {
    exact(
        expand(
            Value::from("a"),
            Some(Value::from("b")),
            Step::from(5),
            Options::new(),
        ),
        &[s("a")],
    );
}

#[test]
fn letter_regex_equal_bounds_is_single_char() {
    let mut o = Options::new();
    o.to_regex = true;
    regex_eq(
        expand(Value::from("a"), Some(Value::from("a")), Step::None, o),
        "a",
    );
}

#[test]
fn letter_regex_with_step_ignores_wrap() {
    // The lettered step regex path always returns the plain join.
    let mut o = Options::new();
    o.to_regex = true;
    o.wrap = true;
    o.capture = true;
    regex_eq(
        expand(Value::from("a"), Some(Value::from("z")), Step::from(3), o),
        "a|d|g|j|m|p|s|v|y",
    );
}

#[test]
fn wrap_without_to_regex_is_noop() {
    let mut o = Options::new();
    o.wrap = true;
    exact(
        expand(Value::from(1), Some(Value::from(3)), Step::None, o),
        &[n(1.0), n(2.0), n(3.0)],
    );
}

#[test]
fn negative_zero_does_not_leak_into_regex() {
    let mut o = Options::new();
    o.to_regex = true;
    let r = expand(Value::from(-5), Some(Value::from(5)), Step::None, o);
    if let FillResult::Regex(src) = r {
        assert!(!src.contains("-0"), "pattern leaked -0: {src}");
    } else {
        panic!("expected a regex");
    }
}
