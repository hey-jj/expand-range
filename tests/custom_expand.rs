//! Transform callback passed as the step argument.

mod common;

use common::{exact, n, s};
use expand_range::{expand, Item, Options, Step, Value};

#[test]
fn current_value_is_first_param() {
    // Identity transform returns the numeric value unchanged.
    let r = expand(
        Value::from(1),
        Some(Value::from(5)),
        Step::Func(Box::new(|v, _i| v)),
        Options::new(),
    );
    exact(r, &[n(1.0), n(2.0), n(3.0), n(4.0), n(5.0)]);
}

#[test]
fn character_code_for_non_integers() {
    // Letter mode passes the character code as the value.
    let r = expand(
        Value::from("a"),
        Some(Value::from("e")),
        Step::Func(Box::new(|v, _i| {
            let code = match v {
                Item::Num(n) => n as u32,
                Item::Str(_) => 0,
            };
            Item::Str(char::from_u32(code).unwrap().to_string())
        })),
        Options::new(),
    );
    exact(r, &[s("a"), s("b"), s("c"), s("d"), s("e")]);
}

#[test]
fn transform_can_pad() {
    let r = expand(
        Value::from("01"),
        Some(Value::from("05")),
        Step::Func(Box::new(|v, _i| {
            let text = v.to_string();
            let width = text.len() + 3;
            let padded = format!("{text:0>width$}");
            Item::Str(padded)
        })),
        Options::new(),
    );
    exact(r, &[s("0001"), s("0002"), s("0003"), s("0004"), s("0005")]);
}

#[test]
fn index_is_second_param() {
    let r = expand(
        Value::from("a"),
        Some(Value::from("e")),
        Step::Func(Box::new(|v, i| {
            let code = match v {
                Item::Num(n) => n as u32,
                Item::Str(_) => 0,
            };
            let ch = char::from_u32(code).unwrap();
            Item::Str(format!("{ch}{i}"))
        })),
        Options::new(),
    );
    exact(r, &[s("a0"), s("b1"), s("c2"), s("d3"), s("e4")]);
}
