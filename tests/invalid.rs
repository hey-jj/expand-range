//! Invalid input returns an empty list when strict mode is off.

mod common;

use common::exact;
use expand_range::{fill, Options, Step, Value};

/// fill(start, end) with no step.
fn f2(start: &str, end: &str) -> expand_range::FillResult {
    fill(
        Value::from(start),
        Some(Value::from(end)),
        Step::None,
        Options::new(),
    )
}

/// fill(start, end, step).
fn f3(start: &str, end: &str, step: &str) -> expand_range::FillResult {
    fill(
        Value::from(start),
        Some(Value::from(end)),
        Step::from(step),
        Options::new(),
    )
}

#[test]
fn invalid_returns_empty() {
    exact(f2("1", "0f"), &[]);
    exact(f3("1", "10", "ff"), &[]);
    exact(f2("1", "10.f"), &[]);
    exact(f2("1", "10f"), &[]);
    exact(f3("1", "20", "2f"), &[]);
    exact(f3("1", "20", "f2"), &[]);
    exact(f2("1", "2f"), &[]);
    exact(f3("1", "2f", "2"), &[]);
    exact(f2("1", "f2"), &[]);
    exact(f2("1", "ff"), &[]);
    exact(f3("1", "ff", "2"), &[]);
    exact(f2("1.1", "2.1"), &[]);
    exact(f2("1.2", "2"), &[]);
    exact(f2("1.20", "2"), &[]);
}
