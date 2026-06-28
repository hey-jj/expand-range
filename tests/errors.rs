//! Strict mode reports typed errors with the established messages.

mod common;

use expand_range::{fill_checked, FillError, Options, Step, Value};

/// Strict options.
fn strict() -> Options {
    let mut o = Options::new();
    o.strict_ranges = true;
    o
}

/// Run the checked entry in strict mode and return the error.
fn err(start: Value, end: Value, step: Step) -> FillError {
    fill_checked(start, Some(end), step, strict()).expect_err("expected an error")
}

#[test]
fn range_error_on_invalid_bounds() {
    assert_eq!(
        err(Value::from("0a"), Value::from("0z"), Step::None),
        FillError::Range("Invalid range arguments: [ '0a', '0z' ]".to_string())
    );
    // Empty start with a step argument and strict mode.
    let e = fill_checked(
        Value::from(""),
        Some(Value::from("*")),
        Step::from(2),
        strict(),
    )
    .expect_err("expected an error");
    assert_eq!(
        e,
        FillError::Range("Invalid range arguments: [ '', '*' ]".to_string())
    );
}

#[test]
fn range_error_on_incompatible_args() {
    assert_eq!(
        err(Value::from("a8"), Value::from(10), Step::None),
        FillError::Range("Invalid range arguments: [ 'a8', 10 ]".to_string())
    );
    assert_eq!(
        err(Value::from(1), Value::from("zz"), Step::None),
        FillError::Range("Invalid range arguments: [ 1, 'zz' ]".to_string())
    );
}

#[test]
fn step_error_on_bad_step() {
    assert_eq!(
        err(Value::from("1"), Value::from("10"), Step::from("z")),
        FillError::Step("Expected step \"z\" to be a number".to_string())
    );
    assert_eq!(
        err(Value::from("a"), Value::from("z"), Step::from("a")),
        FillError::Step("Expected step \"a\" to be a number".to_string())
    );
    assert_eq!(
        err(Value::from("a"), Value::from("z"), Step::from("0a")),
        FillError::Step("Expected step \"0a\" to be a number".to_string())
    );
}
