//! Semantic correctness: compile the generated regex and test membership.

mod common;

use common::is_match;
use expand_range::{Options, Value};

/// Options with to_regex set.
fn regex_opts() -> Options {
    let mut o = Options::new();
    o.to_regex = true;
    o
}

/// Options with to_regex and a step.
fn regex_step(step: f64) -> Options {
    let mut o = Options::new();
    o.to_regex = true;
    o.step = Some(step);
    o
}

/// Test membership for a numeric range against a string input.
fn m(a: i64, b: i64, opts: Options, input: &str) -> bool {
    is_match(Value::from(a), Value::from(b), opts, input)
}

#[test]
fn numbers_ascending() {
    assert!(!m(2, 8, regex_opts(), "10"));
    assert!(m(2, 8, regex_opts(), "3"));
    assert!(m(2, 10, regex_opts(), "10"));
    assert!(m(2, 100, regex_opts(), "10"));
    assert!(!m(2, 100, regex_opts(), "101"));
}

#[test]
fn positive_and_negative() {
    assert!(m(-10, 10, regex_opts(), "10"));
    assert!(m(-10, 10, regex_step(2.0), "10"));
}

#[test]
fn numbers_descending() {
    assert!(m(8, 2, regex_opts(), "2"));
    assert!(m(8, 2, regex_opts(), "8"));
    assert!(!m(8, 2, regex_opts(), "10"));
}

#[test]
fn with_step() {
    assert!(!m(8, 2, regex_step(2.0), "10"));
    assert!(!m(8, 2, regex_step(2.0), "3"));
    assert!(!m(8, 2, regex_step(2.0), "5"));
    assert!(m(8, 2, regex_step(2.0), "8"));
    assert!(!m(2, 8, regex_step(2.0), "10"));
    assert!(!m(2, 8, regex_step(2.0), "3"));
    assert!(m(2, 8, regex_step(2.0), "8"));
}
