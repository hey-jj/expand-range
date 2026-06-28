//! Explicit step argument: numeric and string steps, sign, options.step.

mod common;

use common::{exact, n, s};
use expand_range::{expand, FillResult, Options, Step, Value};

/// expand(start, end, step) where all three are strings.
fn fs(start: &str, end: &str, step: &str) -> FillResult {
    expand(
        Value::from(start),
        Some(Value::from(end)),
        Step::from(step),
        Options::new(),
    )
}

/// expand(num, num, num).
fn fnnn(start: i64, end: i64, step: i64) -> FillResult {
    expand(
        Value::from(start),
        Some(Value::from(end)),
        Step::from(step),
        Options::new(),
    )
}

#[test]
fn increment_with_given_step() {
    exact(
        fs("1", "10", "1"),
        &[
            s("1"),
            s("2"),
            s("3"),
            s("4"),
            s("5"),
            s("6"),
            s("7"),
            s("8"),
            s("9"),
            s("10"),
        ],
    );
    exact(
        fs("1", "10", "2"),
        &[s("1"), s("3"), s("5"), s("7"), s("9")],
    );
    exact(
        fs("0", "1000", "200"),
        &[s("0"), s("200"), s("400"), s("600"), s("800"), s("1000")],
    );
    exact(
        expand(
            Value::from("1"),
            Some(Value::from("10")),
            Step::from(2),
            Options::new(),
        ),
        &[s("1"), s("3"), s("5"), s("7"), s("9")],
    );
    exact(
        fs("1", "20", "2"),
        &[
            s("1"),
            s("3"),
            s("5"),
            s("7"),
            s("9"),
            s("11"),
            s("13"),
            s("15"),
            s("17"),
            s("19"),
        ],
    );
    exact(fs("1", "20", "20"), &[s("1")]);
    // Step sign is ignored.
    exact(
        fs("10", "1", "2"),
        &[s("10"), s("8"), s("6"), s("4"), s("2")],
    );
    exact(
        fs("10", "1", "-2"),
        &[s("10"), s("8"), s("6"), s("4"), s("2")],
    );

    // Number inputs keep number output unless a bound is a string.
    exact(
        expand(
            Value::from(2),
            Some(Value::from(10)),
            Step::from("2"),
            Options::new(),
        ),
        &[n(2.0), n(4.0), n(6.0), n(8.0), n(10.0)],
    );
    exact(
        fnnn(2, 10, 1),
        &[
            n(2.0),
            n(3.0),
            n(4.0),
            n(5.0),
            n(6.0),
            n(7.0),
            n(8.0),
            n(9.0),
            n(10.0),
        ],
    );
    exact(fnnn(2, 10, 2), &[n(2.0), n(4.0), n(6.0), n(8.0), n(10.0)]);
    exact(fnnn(2, 10, 3), &[n(2.0), n(5.0), n(8.0)]);
    exact(fnnn(0, 5, 2), &[n(0.0), n(2.0), n(4.0)]);
    exact(fnnn(5, 0, 2), &[n(5.0), n(3.0), n(1.0)]);
    exact(fnnn(1, 5, 2), &[n(1.0), n(3.0), n(5.0)]);

    // End is a string, so output is strings.
    exact(
        expand(
            Value::from(2),
            Some(Value::from("10")),
            Step::from(2),
            Options::new(),
        ),
        &[s("2"), s("4"), s("6"), s("8"), s("10")],
    );
    exact(
        expand(
            Value::from(2),
            Some(Value::from("10")),
            Step::from(1),
            Options::new(),
        ),
        &[
            s("2"),
            s("3"),
            s("4"),
            s("5"),
            s("6"),
            s("7"),
            s("8"),
            s("9"),
            s("10"),
        ],
    );
    exact(
        expand(
            Value::from(2),
            Some(Value::from("10")),
            Step::from("2"),
            Options::new(),
        ),
        &[s("2"), s("4"), s("6"), s("8"), s("10")],
    );
    exact(
        expand(
            Value::from("2"),
            Some(Value::from(10)),
            Step::from("3"),
            Options::new(),
        ),
        &[s("2"), s("5"), s("8")],
    );
}

#[test]
fn negative_ranges_strings() {
    exact(
        fs("0", "-10", "-2"),
        &[s("0"), s("-2"), s("-4"), s("-6"), s("-8"), s("-10")],
    );
    exact(
        fs("-0", "-10", "-2"),
        &[s("0"), s("-2"), s("-4"), s("-6"), s("-8"), s("-10")],
    );
    exact(
        fs("-1", "-10", "-2"),
        &[s("-1"), s("-3"), s("-5"), s("-7"), s("-9")],
    );
    exact(
        fs("-1", "-10", "2"),
        &[s("-1"), s("-3"), s("-5"), s("-7"), s("-9")],
    );
    exact(
        fs("1", "10", "2"),
        &[s("1"), s("3"), s("5"), s("7"), s("9")],
    );
    exact(
        fs("1", "20", "2"),
        &[
            s("1"),
            s("3"),
            s("5"),
            s("7"),
            s("9"),
            s("11"),
            s("13"),
            s("15"),
            s("17"),
            s("19"),
        ],
    );
    exact(fs("1", "20", "20"), &[s("1")]);
    exact(
        fs("10", "1", "-2"),
        &[s("10"), s("8"), s("6"), s("4"), s("2")],
    );
    exact(
        fs("-10", "0", "2"),
        &[s("-10"), s("-8"), s("-6"), s("-4"), s("-2"), s("0")],
    );
    exact(
        fs("-10", "-0", "2"),
        &[s("-10"), s("-8"), s("-6"), s("-4"), s("-2"), s("0")],
    );
    // Step "0" is truthy, then clamps to 1.
    exact(
        fs("-0", "-10", "0"),
        &[
            s("0"),
            s("-1"),
            s("-2"),
            s("-3"),
            s("-4"),
            s("-5"),
            s("-6"),
            s("-7"),
            s("-8"),
            s("-9"),
            s("-10"),
        ],
    );
    exact(
        fs("0", "-10", "-0"),
        &[
            s("0"),
            s("-1"),
            s("-2"),
            s("-3"),
            s("-4"),
            s("-5"),
            s("-6"),
            s("-7"),
            s("-8"),
            s("-9"),
            s("-10"),
        ],
    );
}

#[test]
fn negative_ranges_numbers() {
    exact(
        fnnn(-10, 0, 2),
        &[n(-10.0), n(-8.0), n(-6.0), n(-4.0), n(-2.0), n(0.0)],
    );
    exact(
        fnnn(-10, -2, 2),
        &[n(-10.0), n(-8.0), n(-6.0), n(-4.0), n(-2.0)],
    );
    exact(
        fnnn(-2, -10, 1),
        &[
            n(-2.0),
            n(-3.0),
            n(-4.0),
            n(-5.0),
            n(-6.0),
            n(-7.0),
            n(-8.0),
            n(-9.0),
            n(-10.0),
        ],
    );
    exact(
        fnnn(0, -10, 2),
        &[n(0.0), n(-2.0), n(-4.0), n(-6.0), n(-8.0), n(-10.0)],
    );
    exact(
        fnnn(-2, -10, 2),
        &[n(-2.0), n(-4.0), n(-6.0), n(-8.0), n(-10.0)],
    );
    exact(fnnn(-2, -10, 3), &[n(-2.0), n(-5.0), n(-8.0)]);
    exact(
        fnnn(-9, 9, 3),
        &[n(-9.0), n(-6.0), n(-3.0), n(0.0), n(3.0), n(6.0), n(9.0)],
    );
}

#[test]
fn negative_zero_passed() {
    // -0 normalizes to 0 in output.
    exact(
        fnnn(-10, 0, 2),
        &[n(-10.0), n(-8.0), n(-6.0), n(-4.0), n(-2.0), n(0.0)],
    );
    exact(
        fnnn(0, -10, 2),
        &[n(0.0), n(-2.0), n(-4.0), n(-6.0), n(-8.0), n(-10.0)],
    );
}

#[test]
fn steps_letters() {
    exact(
        expand(
            Value::from("z"),
            Some(Value::from("a")),
            Step::from(-2),
            Options::new(),
        ),
        &[
            s("z"),
            s("x"),
            s("v"),
            s("t"),
            s("r"),
            s("p"),
            s("n"),
            s("l"),
            s("j"),
            s("h"),
            s("f"),
            s("d"),
            s("b"),
        ],
    );
    exact(
        expand(
            Value::from("a"),
            Some(Value::from("e")),
            Step::from(2),
            Options::new(),
        ),
        &[s("a"), s("c"), s("e")],
    );
    exact(
        expand(
            Value::from("E"),
            Some(Value::from("A")),
            Step::from(2),
            Options::new(),
        ),
        &[s("E"), s("C"), s("A")],
    );
}

#[test]
fn options_step() {
    // An options object in step position carries step and dispatches.
    let opts = || {
        let mut o = Options::new();
        o.step = Some(2.0);
        o
    };
    exact(
        expand(
            Value::from("a"),
            Some(Value::from("e")),
            Step::Opts(opts()),
            Options::new(),
        ),
        &[s("a"), s("c"), s("e")],
    );
    exact(
        expand(
            Value::from("E"),
            Some(Value::from("A")),
            Step::Opts(opts()),
            Options::new(),
        ),
        &[s("E"), s("C"), s("A")],
    );
}
