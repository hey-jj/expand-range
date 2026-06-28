//! Property oracle: a generated pattern must match exactly the integers in range.

mod common;

use common::enumerate;
use expand_range::{expand, FillResult, Options, Step, Value};
use regex::Regex;

/// Build the anchored regex for `[min, max]` once.
fn matcher(min: i64, max: i64) -> Regex {
    let mut opts = Options::new();
    opts.to_regex = true;
    let source = match expand(Value::from(min), Some(Value::from(max)), Step::None, opts) {
        FillResult::Regex(r) => r,
        FillResult::List(_) => panic!("expected regex"),
    };
    Regex::new(&format!("^({source})$")).expect("valid regex")
}

/// For every integer in `[from, to]`, the regex matches iff it lies in `[min, max]`.
fn verify_range(min: i64, max: i64, from: i64, to: i64) {
    let re = matcher(min, max);
    for num in enumerate(from, to) {
        let want = min <= num && num <= max;
        let got = re.is_match(&num.to_string());
        assert_eq!(got, want, "n={num} min={min} max={max}");
    }
}

#[test]
fn equal_numbers() {
    verify_range(1, 1, 0, 100);
    verify_range(65443, 65443, 65000, 66000);
    verify_range(192, 1000, 0, 1000);
}

#[test]
fn large_numbers() {
    verify_range(
        100019999300000,
        100020000300000,
        100019999999999,
        100020000100000,
    );
}

#[test]
fn repeated_digits() {
    verify_range(10331, 20381, 0, 99999);
}

#[test]
fn repeated_zeros() {
    verify_range(10031, 20081, 0, 59999);
    verify_range(10000, 20000, 0, 59999);
}

#[test]
fn zero_one() {
    verify_range(10301, 20101, 0, 99999);
}

#[test]
fn repeated_ones() {
    verify_range(102, 111, 0, 1000);
}

#[test]
fn small_diffs() {
    verify_range(102, 110, 0, 1000);
    verify_range(102, 130, 0, 1000);
}

#[test]
fn random_ranges() {
    verify_range(4173, 7981, 0, 99999);
}

#[test]
fn one_digit_numbers() {
    verify_range(3, 7, 0, 99);
}

#[test]
fn one_digit_at_bounds() {
    verify_range(1, 9, 0, 1000);
}

#[test]
fn power_of_ten() {
    verify_range(1000, 8632, 0, 99999);
}

#[test]
fn varying_lengths() {
    verify_range(1030, 20101, 0, 99999);
    verify_range(13, 8632, 0, 10000);
}

#[test]
fn small_ranges() {
    verify_range(9, 11, 0, 100);
    verify_range(19, 21, 0, 100);
}

#[test]
fn big_ranges() {
    verify_range(90, 98009, 0, 98999);
    verify_range(999, 10000, 1, 20000);
}
