//! Generate numeric and alphabetic ranges with steps and zero padding.
//!
//! This expands a pair of bounds into the values between them, the way shell
//! brace expansion does. `{1..5}` becomes `1 2 3 4 5`. `{a..e}` becomes
//! `a b c d e`. `{01..03}` keeps the padding and yields `01 02 03`. With the
//! `to_regex` option it returns one regex source string that matches every value
//! in the range instead of the list.
//!
//! # Examples
//!
//! ```
//! use expand_range::{range, Item};
//!
//! let r = range(1, 4);
//! assert_eq!(r.expect_list(), &[Item::Num(1), Item::Num(2), Item::Num(3), Item::Num(4)]);
//! ```
//!
//! ```
//! use expand_range::{expand, Value, Step, Options};
//!
//! // String bounds keep their text form, so output stays string-typed.
//! let r = expand(Value::from("a"), Some(Value::from("c")), Step::None, Options::new());
//! assert_eq!(r.to_string_list(), Some(vec!["a".to_string(), "b".to_string(), "c".to_string()]));
//! ```
//!
//! # Behavior
//!
//! - Direction is inferred from the bounds. `expand(5, 1)` descends.
//! - The step magnitude is used. A step of `-2` and `2` behave the same.
//! - Leading zeros on any bound or the step pad every output to a common width.
//! - Bounds may be numbers or single characters. Letter ranges walk character
//!   codes, so `'a'..'C'` passes through ASCII punctuation.
//! - Invalid input returns an empty list. Set `strict_ranges` and call
//!   `expand_checked` to get an error instead.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod options;
mod regex_range;
mod value;

pub use options::{Options, Step, Transform};
pub use value::{FillResult, Item, Value};

use regex_range::{to_regex_range, RegexOptions};
use value::num_to_string;

/// An error returned when `strict_ranges` is set and the input is invalid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FillError {
    /// The bounds do not form a valid range.
    ///
    /// The message names both bounds, with strings single quoted, as in
    /// `Invalid range arguments: [ 'a', 'b' ]`.
    Range(String),
    /// The step is not a number.
    Step(String),
}

impl std::fmt::Display for FillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FillError::Range(m) | FillError::Step(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for FillError {}

impl FillResult {
    /// Collect the list as plain strings, or `None` on a regex result.
    pub fn to_string_list(&self) -> Option<Vec<String>> {
        Some(self.as_list()?.iter().map(|i| i.to_string()).collect())
    }
}

/// Render a bound for error messages.
///
/// Strings are single quoted. Numbers print bare.
fn inspect(value: &Value) -> String {
    match value {
        Value::Num(n) => n.to_string(),
        Value::Str(s) => format!("'{s}'"),
    }
}

/// Build the range-error message for a pair of bounds.
fn range_error(start: &Value, end: &Value) -> FillError {
    FillError::Range(format!(
        "Invalid range arguments: [ {}, {} ]",
        inspect(start),
        inspect(end)
    ))
}

/// Outcome of an invalid range: error under strict mode, else an empty list.
fn invalid_range(start: &Value, end: &Value, strict: bool) -> Result<FillResult, FillError> {
    if strict {
        Err(range_error(start, end))
    } else {
        Ok(FillResult::List(vec![]))
    }
}

/// Outcome of an invalid step: error under strict mode, else an empty list.
fn invalid_step(step: &str, strict: bool) -> Result<FillResult, FillError> {
    if strict {
        Err(FillError::Step(format!(
            "Expected step \"{step}\" to be a number"
        )))
    } else {
        Ok(FillResult::List(vec![]))
    }
}

/// True when a numeric string has a meaningful leading zero.
///
/// `"01"` and `"007"` are padded. `"0"` and `"-0"` are not. The leading sign is
/// stripped first.
fn zeros(input: &str) -> bool {
    let value = input.strip_prefix('-').unwrap_or(input);
    if value == "0" {
        return false;
    }
    value.starts_with('0')
}

/// Left-pad a value to `max_len` with zeros, preserving a leading sign.
///
/// Never truncates. Returns a string when `to_number` is false. When `max_len`
/// is zero, the input passes through unchanged.
fn pad(input: Item, max_len: usize, to_number: bool) -> Item {
    let mut text = input.to_string();
    if max_len > 0 {
        let dash = text.starts_with('-');
        if dash {
            text.remove(0);
        }
        let target = if dash {
            max_len.saturating_sub(1)
        } else {
            max_len
        };
        while text.len() < target {
            text.insert(0, '0');
        }
        if dash {
            text.insert(0, '-');
        }
        // Padding ran, so the result is textual.
        return Item::Str(text);
    }
    if !to_number {
        return Item::Str(text);
    }
    input
}

/// Left-pad a value to exactly `max_len` characters, preserving a leading sign.
///
/// Used on the regex path. Never converts to a number.
fn to_max_len(input: &str, max_len: usize) -> String {
    let (negative, mut body) = match input.strip_prefix('-') {
        Some(rest) => (true, rest.to_string()),
        None => (false, input.to_string()),
    };
    let target = if negative {
        max_len.saturating_sub(1)
    } else {
        max_len
    };
    while body.len() < target {
        body.insert(0, '0');
    }
    if negative {
        format!("-{body}")
    } else {
        body
    }
}

/// True when either bound is a string, or `stringify` is set.
fn stringify_flag(start: &Value, end: &Value, opts: &Options) -> bool {
    matches!(start, Value::Str(_)) || matches!(end, Value::Str(_)) || opts.stringify
}

/// Partitioned absolute values for the stepped regex path.
struct Parts {
    negatives: Vec<i64>,
    positives: Vec<i64>,
}

/// Build the alternation for a stepped numeric regex range.
fn to_sequence(mut parts: Parts, opts: &Options, max_len: usize) -> String {
    parts.negatives.sort_unstable();
    parts.positives.sort_unstable();

    let prefix = if opts.capture { "" } else { "?:" };
    let positives = if parts.positives.is_empty() {
        String::new()
    } else {
        parts
            .positives
            .iter()
            .map(|v| to_max_len(&v.to_string(), max_len))
            .collect::<Vec<_>>()
            .join("|")
    };
    let negatives = if parts.negatives.is_empty() {
        String::new()
    } else {
        let joined = parts
            .negatives
            .iter()
            .map(|v| to_max_len(&v.to_string(), max_len))
            .collect::<Vec<_>>()
            .join("|");
        format!("-({prefix}{joined})")
    };

    let result = if !positives.is_empty() && !negatives.is_empty() {
        format!("{positives}|{negatives}")
    } else if positives.is_empty() {
        negatives
    } else {
        positives
    };

    if opts.wrap {
        format!("({prefix}{result})")
    } else {
        result
    }
}

/// Escape one character so it matches itself as a regex literal.
fn escape_char(c: char) -> String {
    if "\\^$.|?*+()[]{}".contains(c) {
        format!("\\{c}")
    } else {
        c.to_string()
    }
}

/// Regex source for a letter range with step one.
///
/// Returns a single character when the bounds are equal, else a class `[a-z]`.
fn letter_range(a: i64, b: i64) -> String {
    let start = char::from_u32(a as u32).unwrap_or('\u{0}');
    if a == b {
        return escape_char(start);
    }
    let stop = char::from_u32(b as u32).unwrap_or('\u{0}');
    format!("[{}-{}]", escape_char(start), escape_char(stop))
}

/// Regex source for a list of literal members, joined with `|`.
fn to_regex_array(members: &[String], wrap: bool, capture: bool) -> String {
    let joined = members.join("|");
    if wrap {
        let prefix = if capture { "" } else { "?:" };
        format!("({prefix}{joined})")
    } else {
        joined
    }
}

/// Default formatter for a numeric element.
fn number_format(value: i64, to_number: bool) -> Item {
    if to_number {
        Item::num(value)
    } else {
        Item::Str(value.to_string())
    }
}

/// Generate a numeric range.
fn fill_numbers(
    start: &Value,
    end: &Value,
    step_text: &str,
    step_value: f64,
    opts: &Options,
) -> Result<FillResult, FillError> {
    let a_raw = start.to_number().unwrap_or(f64::NAN);
    let b_raw = end.to_number().unwrap_or(f64::NAN);

    if a_raw.fract() != 0.0 || b_raw.fract() != 0.0 || !a_raw.is_finite() || !b_raw.is_finite() {
        return invalid_range(start, end, opts.strict_ranges);
    }

    // Negative zero is normalized to zero by integer math below.
    let a = a_raw as i64;
    let b = b_raw as i64;

    let descending = a > b;
    let start_string = start.as_text();
    let end_string = end.as_text();
    // The step keeps its original text so its leading zeros drive padding.
    let step_string = step_text.to_string();
    let step = step_value.abs().max(1.0) as i64;

    let padded = zeros(&start_string) || zeros(&end_string) || zeros(&step_string);
    let max_len = if padded {
        start_string
            .len()
            .max(end_string.len())
            .max(step_string.len())
    } else {
        0
    };
    let to_number = !padded && !stringify_flag(start, end, opts);

    if opts.to_regex && step == 1 {
        let lo = to_max_len(&start_string, max_len);
        let hi = to_max_len(&end_string, max_len);
        // The step-one path forwards the user's wrap and capture into the
        // interval compiler, which does its own wrapping.
        let ro = RegexOptions {
            relax_zeros: !opts.strict_zeros,
            shorthand: opts.shorthand,
            capture: opts.capture,
            wrap: opts.wrap,
        };
        // A bound wider than i64 cannot be compiled, so treat it as invalid.
        return match to_regex_range(&lo, &hi, &ro) {
            Some(source) => Ok(FillResult::Regex(source)),
            None => invalid_range(start, end, opts.strict_ranges),
        };
    }

    let mut parts = Parts {
        negatives: vec![],
        positives: vec![],
    };
    let mut range: Vec<Item> = vec![];
    let mut current = a;
    let mut index = 0usize;

    loop {
        let in_range = if descending {
            current >= b
        } else {
            current <= b
        };
        if !in_range {
            break;
        }
        if opts.to_regex && step > 1 {
            if current < 0 {
                parts.negatives.push(current.abs());
            } else {
                parts.positives.push(current.abs());
            }
        } else {
            let formatted = match &opts.transform {
                Some(f) => f(Item::num(current), index),
                None => number_format(current, to_number),
            };
            range.push(pad(formatted, max_len, to_number));
        }
        let next = if descending {
            current.checked_sub(step)
        } else {
            current.checked_add(step)
        };
        current = match next {
            Some(v) => v,
            None => break,
        };
        index += 1;
    }

    if opts.to_regex {
        // step is always greater than one here. The step-one regex path returned
        // from the fast path above, and step is clamped to at least one.
        return Ok(FillResult::Regex(to_sequence(parts, opts, max_len)));
    }

    Ok(FillResult::List(range))
}

/// Generate a letter range over character codes.
fn fill_letters(
    start: &Value,
    end: &Value,
    step: i64,
    opts: &Options,
) -> Result<FillResult, FillError> {
    let start_text = start.as_text();
    let end_text = end.as_text();
    let start_bad = !start.is_number() && start_text.chars().count() > 1;
    let end_bad = !end.is_number() && end_text.chars().count() > 1;
    if start_bad || end_bad {
        return invalid_range(start, end, opts.strict_ranges);
    }

    let a0 = start_text.chars().next().map(|c| c as i64).unwrap_or(0);
    let b0 = end_text.chars().next().map(|c| c as i64).unwrap_or(0);

    let descending = a0 > b0;
    let min = a0.min(b0);
    let max = a0.max(b0);

    if opts.to_regex && step == 1 {
        return Ok(FillResult::Regex(letter_range(min, max)));
    }

    let mut range: Vec<Item> = vec![];
    let mut current = a0;
    let mut index = 0usize;
    loop {
        let in_range = if descending {
            current >= b0
        } else {
            current <= b0
        };
        if !in_range {
            break;
        }
        let item = match &opts.transform {
            Some(f) => f(Item::num(current), index),
            None => Item::Str(
                char::from_u32(current as u32)
                    .map(|c| c.to_string())
                    .unwrap_or_default(),
            ),
        };
        range.push(item);
        current = if descending {
            current - step
        } else {
            current + step
        };
        index += 1;
    }

    if opts.to_regex {
        // Letter regex output is always the plain join. wrap and capture do not
        // apply on this path.
        let members: Vec<String> = range
            .iter()
            .map(|i| i.to_string().chars().map(escape_char).collect())
            .collect();
        return Ok(FillResult::Regex(to_regex_array(&members, false, false)));
    }

    Ok(FillResult::List(range))
}

/// Expand a range between two bounds.
///
/// This is the full API. `start` and `end` are numeric or string bounds. `step`
/// is the overloaded step argument: absent, a number, a numeric string, a
/// transform function, or an options object. `options` carries the rest.
///
/// Returns a list of items, or a regex source string when `to_regex` is set.
/// Invalid input yields an empty list.
///
/// This never returns an error and never panics. `strict_ranges` has no effect
/// here. To observe strict-mode errors, set `strict_ranges` and call
/// [`expand_checked`].
pub fn expand(start: Value, end: Option<Value>, step: Step, options: Options) -> FillResult {
    expand_checked(start, end, step, options).unwrap_or(FillResult::List(vec![]))
}

/// Like [`expand`], but returns the strict-mode error instead of swallowing it.
///
/// When `strict_ranges` is set, invalid bounds or a bad step produce
/// [`FillError`]. Otherwise this returns `Ok` with an empty list for invalid
/// input, matching [`expand`].
pub fn expand_checked(
    start: Value,
    end: Option<Value>,
    step: Step,
    options: Options,
) -> Result<FillResult, FillError> {
    // Single argument: a valid lone value returns itself.
    if end.is_none() {
        if start.is_valid() {
            return Ok(FillResult::List(vec![value_to_item(&start)]));
        }
        // Fall through with a placeholder end to reach the invalid path.
        return invalid_range(&start, &Value::Str(String::new()), options.strict_ranges);
    }
    let end = end.unwrap();

    if !start.is_valid() || !end.is_valid() {
        return invalid_range(&start, &end, options.strict_ranges);
    }

    // Step-position dispatch.
    match step {
        Step::Func(f) => {
            // A transform in step position starts a fresh options value that
            // holds only the transform. Any prior options are dropped and the
            // step resets to one.
            let mut opts = Options::new();
            opts.transform = Some(f);
            finish_with_step(start, end, StepValue::Num(1.0), opts)
        }
        // The object becomes options with step reset to its own field or one.
        Step::Opts(opts) => dispatch_with_options(start, end, opts),
        Step::None => dispatch_with_options(start, end, options),
        Step::Num(n) => resolve_numeric_step(start, end, Some(n), None, options),
        Step::Str(s) => resolve_numeric_step(start, end, None, Some(s), options),
    }
}

/// Resolve the step when no positional step was given, then dispatch.
fn dispatch_with_options(
    start: Value,
    end: Value,
    mut opts: Options,
) -> Result<FillResult, FillError> {
    if opts.capture {
        opts.wrap = true;
    }
    // step = step || opts.step || 1, with no positional step.
    let resolved = fallback_step(&opts);
    finish_with_step(start, end, resolved, opts)
}

/// Resolve a positional numeric or string step, then dispatch.
fn resolve_numeric_step(
    start: Value,
    end: Value,
    num: Option<f64>,
    text: Option<String>,
    mut opts: Options,
) -> Result<FillResult, FillError> {
    if opts.capture {
        opts.wrap = true;
    }

    // step = step || opts.step || 1. A numeric 0 is falsy. A "0" string is
    // truthy and kept. An empty string is falsy.
    let resolved: StepValue = match (&num, &text) {
        (Some(n), _) => {
            if *n == 0.0 {
                fallback_step(&opts)
            } else {
                StepValue::Num(*n)
            }
        }
        (_, Some(s)) => {
            if s.is_empty() {
                fallback_step(&opts)
            } else {
                StepValue::Str(s.clone())
            }
        }
        (None, None) => fallback_step(&opts),
    };

    // isNumber(step) check.
    let step_is_number = match &resolved {
        StepValue::Num(n) => n.is_finite() && n.fract() == 0.0,
        StepValue::Str(s) => value::js_coerce(s).is_some_and(|n| n.is_finite() && n.fract() == 0.0),
    };

    if !step_is_number {
        // step != null and not an object -> invalid step. Here it is always a
        // number or string, never null, so this branch always reports.
        let label = match &resolved {
            StepValue::Num(n) => num_to_string(*n),
            StepValue::Str(s) => s.clone(),
        };
        return invalid_step(&label, opts.strict_ranges);
    }

    finish_with_step(start, end, resolved, opts)
}

/// The step value after the `||` fallback chain. Keeps the text form so a
/// padded step like `"03"` still drives output width.
enum StepValue {
    Num(f64),
    Str(String),
}

impl StepValue {
    /// The text used for the `String(step)` length and padding checks.
    fn text(&self) -> String {
        match self {
            StepValue::Num(n) => num_to_string(*n),
            StepValue::Str(s) => s.clone(),
        }
    }

    /// The numeric magnitude source.
    fn value(&self) -> f64 {
        match self {
            StepValue::Num(n) => *n,
            StepValue::Str(s) => value::js_coerce(s).unwrap_or(1.0),
        }
    }
}

/// The fallback when a positional step is falsy: options.step, else one.
fn fallback_step(opts: &Options) -> StepValue {
    match opts.step.filter(|n| *n != 0.0) {
        Some(n) => StepValue::Num(n),
        None => StepValue::Num(1.0),
    }
}

/// Final numbers-versus-letters split with a resolved step.
fn finish_with_step(
    start: Value,
    end: Value,
    step: StepValue,
    opts: Options,
) -> Result<FillResult, FillError> {
    if start.is_number() && end.is_number() {
        fill_numbers(&start, &end, &step.text(), step.value(), &opts)
    } else {
        let letter_step = step.value().abs().max(1.0) as i64;
        fill_letters(&start, &end, letter_step, &opts)
    }
}

/// Convert a bound into the item the single-argument path returns unchanged.
fn value_to_item(value: &Value) -> Item {
    match value {
        Value::Num(n) => Item::Num(*n),
        Value::Str(s) => Item::Str(s.clone()),
    }
}

/// Expand an integer range with default options.
///
/// Convenience over [`expand`] for the common numeric case.
///
/// ```
/// use expand_range::{range, Item};
/// assert_eq!(range(2, 5).expect_list(), &[Item::Num(2), Item::Num(3), Item::Num(4), Item::Num(5)]);
/// ```
pub fn range(start: i64, end: i64) -> FillResult {
    expand(
        Value::from(start),
        Some(Value::from(end)),
        Step::None,
        Options::new(),
    )
}
