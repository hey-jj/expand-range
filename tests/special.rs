//! Negative-zero handling and sign-aware padding.

mod common;

use common::{exact, n, s};
use expand_range::{fill, FillResult, Options, Step, Value};

/// fill(start, end, step) with string bounds and string step.
fn fs(start: &str, end: &str, step: &str) -> FillResult {
    fill(
        Value::from(start),
        Some(Value::from(end)),
        Step::from(step),
        Options::new(),
    )
}

/// fill(start, end, num_step) with string bounds.
fn fsn(start: &str, end: &str, step: i64) -> FillResult {
    fill(
        Value::from(start),
        Some(Value::from(end)),
        Step::from(step),
        Options::new(),
    )
}

/// fill(num, num, num).
fn fnnn(start: i64, end: i64, step: i64) -> FillResult {
    fill(
        Value::from(start),
        Some(Value::from(end)),
        Step::from(step),
        Options::new(),
    )
}

#[test]
fn negative_zero() {
    exact(
        fs("-5", "-0", "-1"),
        &[s("-5"), s("-4"), s("-3"), s("-2"), s("-1"), s("0")],
    );
    exact(fsn("1", "-0", 1), &[s("1"), s("0")]);
    exact(fsn("1", "-0", 0), &[s("1"), s("0")]);
    exact(fs("1", "-0", "0"), &[s("1"), s("0")]);
    exact(fs("1", "-0", "1"), &[s("1"), s("0")]);
    exact(fs("-0", "-0", "1"), &[s("0")]);
    exact(fs("-0", "0", "1"), &[s("0")]);
    exact(
        fs("-0", "5", "1"),
        &[s("0"), s("1"), s("2"), s("3"), s("4"), s("5")],
    );
    exact(
        fill(
            Value::from(-0_i64),
            Some(Value::from(5)),
            Step::None,
            Options::new(),
        ),
        &[n(0.0), n(1.0), n(2.0), n(3.0), n(4.0), n(5.0)],
    );
    exact(fnnn(5, -0, 5), &[n(5.0), n(0.0)]);
    exact(fnnn(5, -0, 2), &[n(5.0), n(3.0), n(1.0)]);
    exact(fnnn(0, 5, 2), &[n(0.0), n(2.0), n(4.0)]);
}

#[test]
fn adjust_padding_for_negative_numbers() {
    exact(
        fill(
            Value::from("-01"),
            Some(Value::from("5")),
            Step::None,
            Options::new(),
        ),
        &[
            s("-01"),
            s("000"),
            s("001"),
            s("002"),
            s("003"),
            s("004"),
            s("005"),
        ],
    );
}
