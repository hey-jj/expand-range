//! Shared helpers for the conformance suite.
//!
//! `exact` compares item type and value, so a number is never equal to its
//! string form.

#![allow(dead_code)]

use expand_range::{expand, expand_checked, FillError, FillResult, Item, Options, Step, Value};

/// Build a numeric item.
pub fn n(x: i64) -> Item {
    Item::Num(x)
}

/// Build a string item.
pub fn s(x: &str) -> Item {
    Item::Str(x.to_string())
}

/// Assert a list result equals the expected items, type and value.
#[track_caller]
pub fn exact(actual: FillResult, expected: &[Item]) {
    match actual {
        FillResult::List(items) => {
            assert_eq!(items.len(), expected.len(), "length mismatch: {items:?}");
            for (i, (a, e)) in items.iter().zip(expected.iter()).enumerate() {
                assert_eq!(a, e, "element {i} mismatch");
            }
        }
        FillResult::Regex(r) => panic!("expected a list, got regex {r:?}"),
    }
}

/// Assert a regex result equals the expected source string.
#[track_caller]
pub fn regex_eq(actual: FillResult, expected: &str) {
    match actual {
        FillResult::Regex(r) => assert_eq!(r, expected),
        FillResult::List(items) => panic!("expected a regex, got list {items:?}"),
    }
}

/// Run expand with non-strict default options.
pub fn run(start: Value, end: Option<Value>, step: Step) -> FillResult {
    expand(start, end, step, Options::new())
}

/// Run expand with the given options.
pub fn run_opts(start: Value, end: Option<Value>, step: Step, opts: Options) -> FillResult {
    expand(start, end, step, opts)
}

/// Run the checked entry, returning the strict-mode error if any.
pub fn run_checked(
    start: Value,
    end: Option<Value>,
    step: Step,
    opts: Options,
) -> Result<FillResult, FillError> {
    expand_checked(start, end, step, opts)
}

/// Build a regex from a numeric range and test a candidate against it.
///
/// Mirrors `new RegExp('^(' + expand(...) + ')$').test(input)`.
pub fn is_match(start: Value, end: Value, opts: Options, input: &str) -> bool {
    let source = match expand(start, Some(end), Step::None, opts) {
        FillResult::Regex(r) => r,
        FillResult::List(_) => panic!("expected a regex result"),
    };
    let re = regex::Regex::new(&format!("^({source})$")).expect("valid regex");
    re.is_match(input)
}

/// Naive inclusive numeric enumerator, used by the match-verification oracle.
pub fn enumerate(start: i64, stop: i64) -> Vec<i64> {
    let mut out = vec![];
    let mut i = start;
    while i <= stop {
        out.push(i);
        i += 1;
    }
    out
}
