//! Input and output value types plus JavaScript-style numeric coercion.
//!
//! A range bound is a number or a string. The library keeps the raw string form
//! because zero padding and length depend on the exact text the caller passed.

use std::fmt;

/// A range bound. Callers pass numbers or strings interchangeably.
///
/// The numeric domain is integers, so a numeric bound is `i64`. The string form
/// is preserved so that leading zeros and text length survive into padding and
/// regex generation.
///
/// A string bound is coerced to a number for the numeric path. The accepted
/// forms are a decimal integer, an optional leading sign, leading zeros, and a
/// decimal or exponent part that still resolves to a whole number. Hex, octal,
/// and binary literals such as `"0x10"` are not accepted and produce an empty
/// range.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Value {
    /// A numeric bound such as `5` or `-10`.
    Num(i64),
    /// A string bound such as `"01"`, `"a"`, or `"-0"`.
    Str(String),
}

impl Value {
    /// The text used for length and padding decisions.
    pub(crate) fn as_text(&self) -> String {
        match self {
            Value::Num(n) => n.to_string(),
            Value::Str(s) => s.clone(),
        }
    }

    /// True when the value is a non-empty string or any number.
    ///
    /// Empty strings are rejected.
    pub(crate) fn is_valid(&self) -> bool {
        match self {
            Value::Num(_) => true,
            Value::Str(s) => !s.is_empty(),
        }
    }

    /// True when the value coerces to an integer.
    ///
    /// A numeric bound is always an integer. A string bound coerces the way
    /// JavaScript `Number.isInteger(+value)` does.
    pub(crate) fn is_number(&self) -> bool {
        match self {
            Value::Num(_) => true,
            Value::Str(s) => js_coerce(s).is_some_and(|n| n.is_finite() && n.fract() == 0.0),
        }
    }

    /// The integer view. String bounds coerce through JavaScript `Number()`.
    pub(crate) fn to_i64(&self) -> Option<i64> {
        match self {
            Value::Num(n) => Some(*n),
            Value::Str(s) => {
                let n = js_coerce(s)?;
                if n.is_finite() && n.fract() == 0.0 {
                    Some(n as i64)
                } else {
                    None
                }
            }
        }
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
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
/// to strings. The distinction matters: `1` and `"1"` are not equal. The
/// numeric element is `i64`, so `Item` derives `Eq` and `Hash` and works as a
/// map key or set member.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Item {
    /// A numeric element.
    Num(i64),
    /// A string element.
    Str(String),
}

impl Item {
    /// Build a numeric item.
    pub(crate) fn num(n: i64) -> Self {
        Item::Num(n)
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::Num(n) => write!(f, "{n}"),
            Item::Str(s) => write!(f, "{s}"),
        }
    }
}

/// The result of an expand call.
#[derive(Clone, Debug, PartialEq)]
pub enum FillResult {
    /// A list of range elements.
    List(Vec<Item>),
    /// A regex source string, produced when `to_regex` is set.
    Regex(String),
}

impl FillResult {
    /// Borrow the list, or `None` when the result is a regex.
    ///
    /// The variant depends on whether `to_regex` was set, so prefer this over
    /// the panicking [`expect_list`](Self::expect_list).
    pub fn as_list(&self) -> Option<&[Item]> {
        match self {
            FillResult::List(items) => Some(items),
            FillResult::Regex(_) => None,
        }
    }

    /// Borrow the regex source, or `None` when the result is a list.
    pub fn as_regex(&self) -> Option<&str> {
        match self {
            FillResult::Regex(s) => Some(s),
            FillResult::List(_) => None,
        }
    }

    /// Take the list, or `None` when the result is a regex.
    ///
    /// Consumes the result so the items move out without a clone.
    pub fn into_list(self) -> Option<Vec<Item>> {
        match self {
            FillResult::List(items) => Some(items),
            FillResult::Regex(_) => None,
        }
    }

    /// Take the regex source, or `None` when the result is a list.
    pub fn into_regex(self) -> Option<String> {
        match self {
            FillResult::Regex(s) => Some(s),
            FillResult::List(_) => None,
        }
    }

    /// Borrow the list, or panic when the result is a regex.
    ///
    /// Convenience for tests and call sites that statically know the variant.
    /// The `expect` name marks the panic. Use [`as_list`](Self::as_list) to
    /// handle both variants.
    pub fn expect_list(&self) -> &[Item] {
        self.as_list().expect("expected a list result, got a regex")
    }

    /// Borrow the regex source, or panic when the result is a list.
    ///
    /// The `expect` name marks the panic. Use [`as_regex`](Self::as_regex) to
    /// handle both variants.
    pub fn expect_regex(&self) -> &str {
        self.as_regex()
            .expect("expected a regex result, got a list")
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
