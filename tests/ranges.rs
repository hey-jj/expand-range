//! Basic range generation: letters, alphanumeric walks, numbers.

mod common;

use common::{exact, n, s};
use expand_range::{fill, Options, Step, Value};

/// fill(start, end) with no step and default options.
fn f2(start: &str, end: &str) -> expand_range::FillResult {
    fill(
        Value::from(start),
        Some(Value::from(end)),
        Step::None,
        Options::new(),
    )
}

/// fill(start) single argument.
fn f1(start: Value) -> expand_range::FillResult {
    fill(start, None, Step::None, Options::new())
}

#[test]
fn increment_alphabetical_ranges() {
    exact(f1(Value::from("a")), &[s("a")]);
    exact(f2("a", "a"), &[s("a")]);
    exact(f2("a", "b"), &[s("a"), s("b")]);
    exact(f2("a", "e"), &[s("a"), s("b"), s("c"), s("d"), s("e")]);
    exact(f2("A", "E"), &[s("A"), s("B"), s("C"), s("D"), s("E")]);
}

#[test]
fn decrement_alphabetical_ranges() {
    exact(f2("E", "A"), &[s("E"), s("D"), s("C"), s("B"), s("A")]);
    exact(
        f2("a", "C"),
        &[
            s("a"),
            s("`"),
            s("_"),
            s("^"),
            s("]"),
            s("\\"),
            s("["),
            s("Z"),
            s("Y"),
            s("X"),
            s("W"),
            s("V"),
            s("U"),
            s("T"),
            s("S"),
            s("R"),
            s("Q"),
            s("P"),
            s("O"),
            s("N"),
            s("M"),
            s("L"),
            s("K"),
            s("J"),
            s("I"),
            s("H"),
            s("G"),
            s("F"),
            s("E"),
            s("D"),
            s("C"),
        ],
    );
    exact(
        f2("z", "m"),
        &[
            s("z"),
            s("y"),
            s("x"),
            s("w"),
            s("v"),
            s("u"),
            s("t"),
            s("s"),
            s("r"),
            s("q"),
            s("p"),
            s("o"),
            s("n"),
            s("m"),
        ],
    );
}

#[test]
fn increment_alphanumeric_ranges() {
    exact(
        f2("9", "B"),
        &[
            s("9"),
            s(":"),
            s(";"),
            s("<"),
            s("="),
            s(">"),
            s("?"),
            s("@"),
            s("A"),
            s("B"),
        ],
    );
    exact(
        f2("A", "10"),
        &[
            s("A"),
            s("@"),
            s("?"),
            s(">"),
            s("="),
            s("<"),
            s(";"),
            s(":"),
            s("9"),
            s("8"),
            s("7"),
            s("6"),
            s("5"),
            s("4"),
            s("3"),
            s("2"),
            s("1"),
        ],
    );
    exact(
        f2("a", "10"),
        &[
            s("a"),
            s("`"),
            s("_"),
            s("^"),
            s("]"),
            s("\\"),
            s("["),
            s("Z"),
            s("Y"),
            s("X"),
            s("W"),
            s("V"),
            s("U"),
            s("T"),
            s("S"),
            s("R"),
            s("Q"),
            s("P"),
            s("O"),
            s("N"),
            s("M"),
            s("L"),
            s("K"),
            s("J"),
            s("I"),
            s("H"),
            s("G"),
            s("F"),
            s("E"),
            s("D"),
            s("C"),
            s("B"),
            s("A"),
            s("@"),
            s("?"),
            s(">"),
            s("="),
            s("<"),
            s(";"),
            s(":"),
            s("9"),
            s("8"),
            s("7"),
            s("6"),
            s("5"),
            s("4"),
            s("3"),
            s("2"),
            s("1"),
        ],
    );
}

#[test]
fn step_alphanumeric_ranges() {
    exact(
        fill(
            Value::from("9"),
            Some(Value::from("B")),
            Step::from(3),
            Options::new(),
        ),
        &[s("9"), s("<"), s("?"), s("B")],
    );
}

#[test]
fn decrement_alphanumeric_ranges() {
    exact(
        f2("C", "9"),
        &[
            s("C"),
            s("B"),
            s("A"),
            s("@"),
            s("?"),
            s(">"),
            s("="),
            s("<"),
            s(";"),
            s(":"),
            s("9"),
        ],
    );
}

#[test]
fn increment_numeric_string_ranges() {
    exact(f1(Value::from("1")), &[s("1")]);
    exact(f2("1", "1"), &[s("1")]);
    exact(f2("1", "2"), &[s("1"), s("2")]);
    exact(
        f2("1", "10"),
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
    exact(f2("1", "3"), &[s("1"), s("2"), s("3")]);
    exact(f2("5", "8"), &[s("5"), s("6"), s("7"), s("8")]);
    exact(
        f2("1", "9"),
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
        ],
    );
}

#[test]
fn increment_numeric_number_ranges() {
    exact(num(1, 3), &[n(1.0), n(2.0), n(3.0)]);
    exact(
        num(1, 9),
        &[
            n(1.0),
            n(2.0),
            n(3.0),
            n(4.0),
            n(5.0),
            n(6.0),
            n(7.0),
            n(8.0),
            n(9.0),
        ],
    );
    exact(num(5, 8), &[n(5.0), n(6.0), n(7.0), n(8.0)]);
}

/// fill(num, num).
fn num(a: i64, b: i64) -> expand_range::FillResult {
    fill(
        Value::from(a),
        Some(Value::from(b)),
        Step::None,
        Options::new(),
    )
}

#[test]
fn combination_of_number_and_string() {
    // start is a string, so output is strings.
    exact(
        fill(
            Value::from("1"),
            Some(Value::from(9_i64)),
            Step::None,
            Options::new(),
        ),
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
        ],
    );
    exact(
        fill(
            Value::from("2"),
            Some(Value::from(5_i64)),
            Step::None,
            Options::new(),
        ),
        &[s("2"), s("3"), s("4"), s("5")],
    );
}

#[test]
fn decrement_numeric_string_ranges() {
    exact(
        f2("0", "-5"),
        &[s("0"), s("-1"), s("-2"), s("-3"), s("-4"), s("-5")],
    );
    exact(
        f2("-1", "-5"),
        &[s("-1"), s("-2"), s("-3"), s("-4"), s("-5")],
    );
}

#[test]
fn decrement_numeric_number_ranges() {
    exact(
        num(-10, -1),
        &[
            n(-10.0),
            n(-9.0),
            n(-8.0),
            n(-7.0),
            n(-6.0),
            n(-5.0),
            n(-4.0),
            n(-3.0),
            n(-2.0),
            n(-1.0),
        ],
    );
    exact(
        num(0, -5),
        &[n(0.0), n(-1.0), n(-2.0), n(-3.0), n(-4.0), n(-5.0)],
    );
}

#[test]
fn string_ranges_positive_and_negative() {
    exact(
        f2("9", "-4"),
        &[
            s("9"),
            s("8"),
            s("7"),
            s("6"),
            s("5"),
            s("4"),
            s("3"),
            s("2"),
            s("1"),
            s("0"),
            s("-1"),
            s("-2"),
            s("-3"),
            s("-4"),
        ],
    );
    exact(
        f2("-5", "5"),
        &[
            s("-5"),
            s("-4"),
            s("-3"),
            s("-2"),
            s("-1"),
            s("0"),
            s("1"),
            s("2"),
            s("3"),
            s("4"),
            s("5"),
        ],
    );
}

#[test]
fn number_ranges_positive_and_negative() {
    exact(
        num(9, -4),
        &[
            n(9.0),
            n(8.0),
            n(7.0),
            n(6.0),
            n(5.0),
            n(4.0),
            n(3.0),
            n(2.0),
            n(1.0),
            n(0.0),
            n(-1.0),
            n(-2.0),
            n(-3.0),
            n(-4.0),
        ],
    );
    exact(
        num(-5, 5),
        &[
            n(-5.0),
            n(-4.0),
            n(-3.0),
            n(-2.0),
            n(-1.0),
            n(0.0),
            n(1.0),
            n(2.0),
            n(3.0),
            n(4.0),
            n(5.0),
        ],
    );
}
