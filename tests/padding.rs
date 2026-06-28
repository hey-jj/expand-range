//! Zero padding of generated values and of regex output.

mod common;

use common::{exact, regex_eq, s};
use expand_range::{fill, FillResult, Options, Step, Value};

/// fill(start, end) with no step.
fn f2(start: &str, end: &str) -> FillResult {
    fill(
        Value::from(start),
        Some(Value::from(end)),
        Step::None,
        Options::new(),
    )
}

/// fill(start, end, step).
fn f3(start: &str, end: &str, step: &str) -> FillResult {
    fill(
        Value::from(start),
        Some(Value::from(end)),
        Step::from(step),
        Options::new(),
    )
}

/// fill(start, end, num_step).
fn f3n(start: &str, end: &str, step: i64) -> FillResult {
    fill(
        Value::from(start),
        Some(Value::from(end)),
        Step::from(step),
        Options::new(),
    )
}

/// fill(start, end, {toRegex:true}).
fn fre(start: &str, end: &str) -> FillResult {
    let mut o = Options::new();
    o.to_regex = true;
    fill(Value::from(start), Some(Value::from(end)), Step::None, o)
}

/// fill(start, end, {toRegex:true, strictZeros:true}).
fn fre_strict(start: &str, end: &str) -> FillResult {
    let mut o = Options::new();
    o.to_regex = true;
    o.strict_zeros = true;
    fill(Value::from(start), Some(Value::from(end)), Step::None, o)
}

#[test]
fn pad_incremented_numbers() {
    exact(f2("01", "03"), &[s("01"), s("02"), s("03")]);
    exact(f2("01", "3"), &[s("01"), s("02"), s("03")]);
    exact(f2("1", "03"), &[s("01"), s("02"), s("03")]);
    exact(f2("0001", "0003"), &[s("0001"), s("0002"), s("0003")]);
    exact(
        f2("-10", "00"),
        &[
            s("-10"),
            s("-09"),
            s("-08"),
            s("-07"),
            s("-06"),
            s("-05"),
            s("-04"),
            s("-03"),
            s("-02"),
            s("-01"),
            s("000"),
        ],
    );
    exact(
        f2("05", "010"),
        &[s("005"), s("006"), s("007"), s("008"), s("009"), s("010")],
    );
    // Spot-check a long padded run.
    let r = f2("05", "100");
    let items = r.list();
    assert_eq!(items.len(), 96);
    assert_eq!(items.first(), Some(&s("005")));
    assert_eq!(items.last(), Some(&s("100")));
    assert_eq!(items[5], s("010"));
}

#[test]
fn pad_decremented_numbers() {
    exact(f2("03", "01"), &[s("03"), s("02"), s("01")]);
    exact(f2("3", "01"), &[s("03"), s("02"), s("01")]);
    exact(f2("003", "1"), &[s("003"), s("002"), s("001")]);
    exact(f2("003", "001"), &[s("003"), s("002"), s("001")]);
    exact(f2("3", "001"), &[s("003"), s("002"), s("001")]);
    exact(f2("03", "001"), &[s("003"), s("002"), s("001")]);
}

#[test]
fn pad_decremented_with_regex() {
    regex_eq(fre("03", "01"), "0?[1-3]");
    regex_eq(fre("3", "01"), "0?[1-3]");
    regex_eq(fre("003", "1"), "0{0,2}[1-3]");
    regex_eq(fre("003", "001"), "0{0,2}[1-3]");
    regex_eq(fre("3", "001"), "0{0,2}[1-3]");
    regex_eq(fre("03", "001"), "0{0,2}[1-3]");
    regex_eq(fre("001", "020"), "0{0,2}[1-9]|0?1[0-9]|0?20");
}

#[test]
fn pad_with_strict_zeros() {
    regex_eq(fre_strict("03", "01"), "0[1-3]");
    regex_eq(fre_strict("3", "01"), "0[1-3]");
    regex_eq(fre_strict("003", "1"), "00[1-3]");
    regex_eq(fre_strict("003", "001"), "00[1-3]");
    regex_eq(fre_strict("3", "001"), "00[1-3]");
    regex_eq(fre_strict("03", "001"), "00[1-3]");
    regex_eq(fre_strict("001", "020"), "00[1-9]|01[0-9]|020");
}

#[test]
fn pad_stepped_numbers() {
    exact(f3("1", "05", "3"), &[s("01"), s("04")]);
    exact(f3("1", "5", "03"), &[s("01"), s("04")]);
    exact(f3("1", "5", "0003"), &[s("0001"), s("0004")]);
    exact(f3("1", "005", "3"), &[s("001"), s("004")]);
    exact(
        f3("00", "1000", "200"),
        &[
            s("0000"),
            s("0200"),
            s("0400"),
            s("0600"),
            s("0800"),
            s("1000"),
        ],
    );
    exact(
        f3("0", "01000", "200"),
        &[
            s("00000"),
            s("00200"),
            s("00400"),
            s("00600"),
            s("00800"),
            s("01000"),
        ],
    );
    exact(f3("001", "5", "3"), &[s("001"), s("004")]);
    exact(
        f3n("02", "10", 2),
        &[s("02"), s("04"), s("06"), s("08"), s("10")],
    );
    exact(
        f3n("002", "10", 2),
        &[s("002"), s("004"), s("006"), s("008"), s("010")],
    );
    exact(
        f3n("002", "010", 2),
        &[s("002"), s("004"), s("006"), s("008"), s("010")],
    );
    exact(
        fill(
            Value::from("-04"),
            Some(Value::from(4)),
            Step::from(2),
            Options::new(),
        ),
        &[s("-04"), s("-02"), s("000"), s("002"), s("004")],
    );
}
