//! Options and the overloaded step argument.

use crate::value::Item;
use std::fmt;

/// A per-element formatter: `(value, index) -> item`.
///
/// For numeric ranges `value` is the numeric element. For letter ranges `value`
/// is the character code as a number. `index` is the zero-based position.
pub type Transform = Box<dyn Fn(Item, usize) -> Item>;

/// Settings that control range generation.
///
/// Every field defaults to off. Set fields directly, or chain the setters from
/// `Options::new()`:
///
/// ```
/// use expand_range::Options;
/// let opts = Options::new().to_regex(true).wrap(true);
/// ```
///
/// `strict_ranges` only changes behavior on [`expand_checked`]. On [`expand`] a
/// strict error is swallowed into an empty list. See the field doc below.
///
/// [`expand`]: crate::expand
/// [`expand_checked`]: crate::expand_checked
#[derive(Default)]
pub struct Options {
    /// Increment to use when the step argument is absent.
    pub step: Option<f64>,
    /// Return errors for invalid input instead of an empty list.
    ///
    /// This only takes effect on [`expand_checked`]. On [`expand`] the error is
    /// swallowed and the result is an empty list, so set this flag and call
    /// [`expand_checked`] to observe the error.
    ///
    /// [`expand`]: crate::expand
    /// [`expand_checked`]: crate::expand_checked
    pub strict_ranges: bool,
    /// Force numeric output to strings.
    pub stringify: bool,
    /// Return a regex source string instead of a list.
    pub to_regex: bool,
    /// Wrap regex alternations in a non-capturing group.
    pub wrap: bool,
    /// Wrap regex alternations in a capturing group.
    ///
    /// Setting this forces `wrap` on during dispatch, so a capturing group
    /// always wraps the whole alternation.
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
    ///
    /// Use it as the entry point for the chainable setters.
    pub fn new() -> Self {
        Options::default()
    }

    /// Set the fallback step used when no step argument is given.
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Set whether invalid input returns an error on [`expand_checked`].
    ///
    /// [`expand_checked`]: crate::expand_checked
    pub fn strict_ranges(mut self, on: bool) -> Self {
        self.strict_ranges = on;
        self
    }

    /// Set whether numeric output is forced to strings.
    pub fn stringify(mut self, on: bool) -> Self {
        self.stringify = on;
        self
    }

    /// Set whether the result is a regex source string instead of a list.
    pub fn to_regex(mut self, on: bool) -> Self {
        self.to_regex = on;
        self
    }

    /// Set whether regex alternations wrap in a non-capturing group.
    pub fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }

    /// Set whether regex alternations wrap in a capturing group.
    ///
    /// This forces `wrap` on during dispatch.
    pub fn capture(mut self, on: bool) -> Self {
        self.capture = on;
        self
    }

    /// Set whether padded regex output forbids optional leading zeros.
    pub fn strict_zeros(mut self, on: bool) -> Self {
        self.strict_zeros = on;
        self
    }

    /// Set whether padded regex output uses `\d` instead of `[0-9]`.
    pub fn shorthand(mut self, on: bool) -> Self {
        self.shorthand = on;
        self
    }

    /// Set the per-element formatter that replaces the default.
    pub fn transform(mut self, f: Transform) -> Self {
        self.transform = Some(f);
        self
    }
}

impl fmt::Debug for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Options")
            .field("step", &self.step)
            .field("strict_ranges", &self.strict_ranges)
            .field("stringify", &self.stringify)
            .field("to_regex", &self.to_regex)
            .field("wrap", &self.wrap)
            .field("capture", &self.capture)
            .field("strict_zeros", &self.strict_zeros)
            .field("shorthand", &self.shorthand)
            .field("transform", &self.transform.as_ref().map(|_| Placeholder))
            .finish()
    }
}

/// Stand-in for the boxed closure fields, which have no useful Debug form.
struct Placeholder;

impl fmt::Debug for Placeholder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<fn>")
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

impl fmt::Debug for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Step::None => f.write_str("None"),
            Step::Num(n) => f.debug_tuple("Num").field(n).finish(),
            Step::Str(s) => f.debug_tuple("Str").field(s).finish(),
            Step::Func(_) => f.debug_tuple("Func").field(&Placeholder).finish(),
            Step::Opts(o) => f.debug_tuple("Opts").field(o).finish(),
        }
    }
}

impl From<i64> for Step {
    fn from(n: i64) -> Self {
        Step::Num(n as f64)
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
