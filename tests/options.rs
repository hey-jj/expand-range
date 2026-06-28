//! stringify, transform, toRegex, wrap, and capture options.

mod common;

use common::{exact, regex_eq, s};
use expand_range::{fill, FillResult, Item, Options, Step, Value};

/// Options with stringify set.
fn stringify_opts() -> Options {
    let mut o = Options::new();
    o.stringify = true;
    o
}

/// Options with to_regex set.
fn regex_opts() -> Options {
    let mut o = Options::new();
    o.to_regex = true;
    o
}

#[test]
fn stringify_casts_to_strings() {
    exact(
        fill(
            Value::from("1"),
            Some(Value::from("10")),
            Step::from("1"),
            stringify_opts(),
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
            s("10"),
        ],
    );
    exact(
        fill(
            Value::from(2),
            Some(Value::from(10)),
            Step::from("2"),
            stringify_opts(),
        ),
        &[s("2"), s("4"), s("6"), s("8"), s("10")],
    );
    exact(
        fill(
            Value::from(2),
            Some(Value::from(10)),
            Step::from(1),
            stringify_opts(),
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
        fill(
            Value::from(2),
            Some(Value::from(10)),
            Step::from(3),
            stringify_opts(),
        ),
        &[s("2"), s("5"), s("8")],
    );
}

#[test]
fn transform_casts_to_strings() {
    let opts = || {
        let mut o = Options::new();
        o.transform = Some(Box::new(|v: Item, _i| Item::Str(v.to_string())));
        o
    };
    exact(
        fill(
            Value::from("1"),
            Some(Value::from("10")),
            Step::from("1"),
            opts(),
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
            s("10"),
        ],
    );
    exact(
        fill(
            Value::from(2),
            Some(Value::from(10)),
            Step::from("2"),
            opts(),
        ),
        &[s("2"), s("4"), s("6"), s("8"), s("10")],
    );
    exact(
        fill(Value::from(2), Some(Value::from(10)), Step::from(1), opts()),
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
        fill(Value::from(2), Some(Value::from(10)), Step::from(3), opts()),
        &[s("2"), s("5"), s("8")],
    );
}

/// fill(num, num, {toRegex:true}).
fn rnn(a: i64, b: i64) -> FillResult {
    fill(
        Value::from(a),
        Some(Value::from(b)),
        Step::None,
        regex_opts(),
    )
}

#[test]
fn regex_numbers_ascending() {
    regex_eq(rnn(2, 8), "[2-8]");
    regex_eq(rnn(2, 10), "[2-9]|10");
    regex_eq(rnn(2, 100), "[2-9]|[1-9][0-9]|100");
}

#[test]
fn regex_positive_and_negative() {
    regex_eq(rnn(-10, 10), "-[1-9]|-?10|[0-9]");
    regex_eq(
        fill(
            Value::from(-10),
            Some(Value::from(10)),
            Step::from(2),
            regex_opts(),
        ),
        "0|2|4|6|8|10|-(?:2|4|6|8|10)",
    );
    regex_eq(
        fill(
            Value::from(-10),
            Some(Value::from(0)),
            Step::from(2),
            regex_opts(),
        ),
        "0|-(?:2|4|6|8|10)",
    );
    regex_eq(
        fill(
            Value::from(-10),
            Some(Value::from(-2)),
            Step::from(2),
            regex_opts(),
        ),
        "-(?:2|4|6|8|10)",
    );
}

#[test]
fn regex_numbers_descending() {
    regex_eq(rnn(8, 2), "[2-8]");
}

#[test]
fn regex_with_step() {
    let opts = || {
        let mut o = Options::new();
        o.to_regex = true;
        o.step = Some(2.0);
        o
    };
    regex_eq(
        fill(Value::from(8), Some(Value::from(2)), Step::None, opts()),
        "2|4|6|8",
    );
    regex_eq(
        fill(Value::from(2), Some(Value::from(8)), Step::None, opts()),
        "2|4|6|8",
    );
}

/// fill(str, str, {toRegex:true}).
fn rss(a: &str, b: &str) -> FillResult {
    fill(
        Value::from(a),
        Some(Value::from(b)),
        Step::None,
        regex_opts(),
    )
}

#[test]
fn regex_zero_padding() {
    regex_eq(rss("002", "008"), "0{0,2}[2-8]");
    regex_eq(rss("02", "08"), "0?[2-8]");
    regex_eq(rss("02", "10"), "0?[2-9]|10");
    regex_eq(rss("002", "100"), "0{0,2}[2-9]|0?[1-9][0-9]|100");
}

#[test]
fn regex_negative_zero_padding() {
    regex_eq(rss("-002", "-100"), "-0{0,3}[2-9]|-0{0,2}[1-9][0-9]|-0?100");
    regex_eq(rss("-02", "-08"), "-0{0,2}[2-8]");
    regex_eq(rss("-02", "-100"), "-0{0,3}[2-9]|-0{0,2}[1-9][0-9]|-0?100");
    regex_eq(
        rss("-02", "100"),
        "-0{0,2}[12]|0{0,2}[0-9]|0?[1-9][0-9]|100",
    );
}

#[test]
fn regex_letters_ascending() {
    regex_eq(rss("a", "b"), "[a-b]");
    regex_eq(rss("A", "b"), "[A-b]");
    regex_eq(rss("Z", "a"), "[Z-a]");
}

#[test]
fn regex_letters_descending() {
    regex_eq(rss("z", "A"), "[A-z]");
}

#[test]
fn wrap_single_condition_not_wrapped() {
    let mut o = Options::new();
    o.to_regex = true;
    o.wrap = true;
    regex_eq(
        fill(Value::from(2), Some(Value::from(8)), Step::None, o),
        "[2-8]",
    );
}

#[test]
fn wrap_multi_in_parentheses() {
    let opts = || {
        let mut o = Options::new();
        o.to_regex = true;
        o.wrap = true;
        o
    };
    regex_eq(
        fill(Value::from(2), Some(Value::from(10)), Step::None, opts()),
        "(?:[2-9]|10)",
    );
    regex_eq(
        fill(Value::from(2), Some(Value::from(100)), Step::None, opts()),
        "(?:[2-9]|[1-9][0-9]|100)",
    );
}

#[test]
fn wrap_positive_and_negative() {
    let opts = || {
        let mut o = Options::new();
        o.to_regex = true;
        o.wrap = true;
        o
    };
    regex_eq(
        fill(
            Value::from(-10),
            Some(Value::from(-2)),
            Step::from(2),
            opts(),
        ),
        "(?:-(?:2|4|6|8|10))",
    );
    regex_eq(
        fill(
            Value::from(-10),
            Some(Value::from(0)),
            Step::from(2),
            opts(),
        ),
        "(?:0|-(?:2|4|6|8|10))",
    );
    regex_eq(
        fill(
            Value::from(-10),
            Some(Value::from(10)),
            Step::from(2),
            opts(),
        ),
        "(?:0|2|4|6|8|10|-(?:2|4|6|8|10))",
    );
    regex_eq(
        fill(Value::from(-10), Some(Value::from(10)), Step::None, opts()),
        "(?:-[1-9]|-?10|[0-9])",
    );
}

#[test]
fn capture_uses_capturing_group() {
    let opts = || {
        let mut o = Options::new();
        o.to_regex = true;
        o.capture = true;
        o
    };
    regex_eq(
        fill(
            Value::from(-10),
            Some(Value::from(10)),
            Step::from(2),
            opts(),
        ),
        "(0|2|4|6|8|10|-(2|4|6|8|10))",
    );
    regex_eq(
        fill(Value::from(-10), Some(Value::from(10)), Step::None, opts()),
        "(-[1-9]|-?10|[0-9])",
    );
}
