//! Options and the overloaded step argument.

use crate::value::Item;

/// A per-element formatter: `(value, index) -> item`.
///
/// For numeric ranges `value` is the numeric element. For letter ranges `value`
/// is the character code as a number. `index` is the zero-based position.
pub type Transform = Box<dyn Fn(Item, usize) -> Item>;

/// Settings that control range generation.
///
/// All fields default to off. Build with `Options::default()` and set what you
/// need, or use the helper constructors on the crate root.
#[derive(Default)]
pub struct Options {
    /// Increment to use when the step argument is absent.
    pub step: Option<f64>,
    /// Throw on invalid input instead of returning an empty list.
    pub strict_ranges: bool,
    /// Force numeric output to strings.
    pub stringify: bool,
    /// Return a regex source string instead of a list.
    pub to_regex: bool,
    /// Wrap regex alternations in a non-capturing group.
    pub wrap: bool,
    /// Wrap regex alternations in a capturing group. Implies `wrap`.
    pub capture: bool,
    /// Forbid optional leading zeros in padded regex output.
    pub strict_zeros: bool,
    /// Emit `\d` instead of `[0-9]` in padded regex output.
    pub shorthand: bool,
    /// Per-element formatter that replaces the default.
    pub transform: Option<Transform>,
}

impl Options {
    /// A fresh options value with everything off.
    pub fn new() -> Self {
        Options::default()
    }
}

/// The step argument, overloaded by JavaScript runtime type.
///
/// A step can be absent, a number, a numeric string, a transform function in
/// step position, or an options object in step position.
pub enum Step {
    /// No step given.
    None,
    /// A numeric step.
    Num(f64),
    /// A string step such as `"2"` or `"-2"`.
    Str(String),
    /// A function passed where a step would go. Becomes `options.transform`.
    Func(Transform),
    /// An options object passed where a step would go. Becomes `options`.
    Opts(Options),
}

impl From<i64> for Step {
    fn from(n: i64) -> Self {
        Step::Num(n as f64)
    }
}

impl From<f64> for Step {
    fn from(n: f64) -> Self {
        Step::Num(n)
    }
}

impl From<&str> for Step {
    fn from(s: &str) -> Self {
        Step::Str(s.to_string())
    }
}

impl From<Options> for Step {
    fn from(o: Options) -> Self {
        Step::Opts(o)
    }
}
