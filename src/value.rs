//! Input and output value types plus JavaScript-style numeric coercion.
//!
//! A range bound is a number or a string. The library keeps the raw string form
//! because zero padding and length depend on the exact text the caller passed.

use std::fmt;

/// A range bound. Callers pass numbers or strings interchangeably.
///
/// The string form is preserved so that leading zeros and text length survive
/// into padding and regex generation.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// A numeric bound such as `5` or `-10`.
    Num(f64),
    /// A string bound such as `"01"`, `"a"`, or `"-0"`.
    Str(String),
}

impl Value {
    /// The text used for length and padding decisions.
    ///
    /// Numbers render the way JavaScript `String(n)` would for the integers this
    /// library handles. `-0` becomes `"0"`.
    pub(crate) fn as_text(&self) -> String {
        match self {
            Value::Num(n) => num_to_string(*n),
            Value::Str(s) => s.clone(),
        }
    }

    /// True when the value is a non-empty string or any number.
    ///
    /// Mirrors the `isValidValue` guard. Empty strings are rejected.
    pub(crate) fn is_valid(&self) -> bool {
        match self {
            Value::Num(_) => true,
            Value::Str(s) => !s.is_empty(),
        }
    }

    /// True when the value coerces to an integer.
    ///
    /// Uses the same coercion as JavaScript `Number.isInteger(+value)`.
    pub(crate) fn is_number(&self) -> bool {
        match self {
            Value::Num(n) => n.is_finite() && n.fract() == 0.0,
            Value::Str(s) => js_coerce(s).is_some_and(|n| n.is_finite() && n.fract() == 0.0),
        }
    }

    /// The numeric view via JavaScript `Number(value)` coercion.
    pub(crate) fn to_number(&self) -> Option<f64> {
        match self {
            Value::Num(n) => Some(*n),
            Value::Str(s) => js_coerce(s),
        }
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Num(n as f64)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Num(n)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Str(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Str(s)
    }
}

/// One produced element of a range.
///
/// Numbers stay numbers unless the range is padded, stringified, or transformed
/// to strings. The distinction matters: `1` and `"1"` are not equal.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    /// A numeric element.
    Num(f64),
    /// A string element.
    Str(String),
}

impl Item {
    /// Build a numeric item, normalizing `-0` to `0` like JavaScript does.
    pub(crate) fn num(n: f64) -> Self {
        Item::Num(if n == 0.0 { 0.0 } else { n })
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::Num(n) => write!(f, "{}", num_to_string(*n)),
            Item::Str(s) => write!(f, "{s}"),
        }
    }
}

/// The result of a fill call.
#[derive(Clone, Debug, PartialEq)]
pub enum FillResult {
    /// A list of range elements.
    List(Vec<Item>),
    /// A regex source string, produced when `to_regex` is set.
    Regex(String),
}

impl FillResult {
    /// Borrow the list, or panic when the result is a regex.
    ///
    /// Test and caller convenience for the common array case.
    pub fn list(&self) -> &[Item] {
        match self {
            FillResult::List(items) => items,
            FillResult::Regex(_) => panic!("expected a list result, got a regex"),
        }
    }

    /// Borrow the regex source, or panic when the result is a list.
    pub fn regex(&self) -> &str {
        match self {
            FillResult::Regex(s) => s,
            FillResult::List(_) => panic!("expected a regex result, got a list"),
        }
    }
}

/// Render a number the way JavaScript `String(n)` does for the integers this
/// library produces. `-0` renders as `"0"`.
pub(crate) fn num_to_string(n: f64) -> String {
    if n == 0.0 {
        return "0".to_string();
    }
    if n.fract() == 0.0 && n.is_finite() {
        // Integer values: print without a decimal point.
        format!("{}", n as i128)
    } else {
        format!("{n}")
    }
}

/// JavaScript unary-plus coercion for strings, limited to the forms this library
/// meets in practice.
///
/// Handles optional surrounding whitespace, an empty or all-whitespace string as
/// `0`, an optional leading sign, decimal digits, and a fractional or exponent
/// part. Returns `None` for anything with stray characters, which maps to JS
/// `NaN`. JS hex, octal, and binary literals are out of scope because no input
/// in the supported range uses them.
pub(crate) fn js_coerce(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return Some(0.0);
    }
    // Rust f64 parsing accepts the decimal and exponent grammar JS uses, and
    // rejects trailing letters. It also accepts a leading sign. Reject the few
    // tokens Rust takes but JS coercion does not.
    match t {
        "inf" | "-inf" | "+inf" | "infinity" | "-infinity" | "+infinity" | "nan" | "NaN" => {
            return None;
        }
        _ => {}
    }
    // Reject a lone sign or forms Rust would otherwise accept oddly.
    t.parse::<f64>().ok()
}
