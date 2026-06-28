# expand-range

Generate numeric and alphabetic ranges with steps and zero padding, the way
shell brace expansion does. `{1..5}` becomes `1 2 3 4 5`. `{a..e}` becomes
`a b c d e`. `{01..03}` keeps the padding and yields `01 02 03`. With the regex
option it returns one regex source string that matches every value in the range.

## Install

```toml
[dependencies]
expand-range = "0.1"
```

## Usage

```rust
use expand_range::{fill_range, Item};

let r = fill_range(1, 4);
assert_eq!(
    r.list(),
    &[Item::Num(1.0), Item::Num(2.0), Item::Num(3.0), Item::Num(4.0)]
);
```

Letters walk character codes:

```rust
use expand_range::{fill, Value, Step, Options};

let r = fill(Value::from("a"), Some(Value::from("c")), Step::None, Options::new());
assert_eq!(r.to_string_list(), vec!["a", "b", "c"]);
```

A regex source string instead of a list:

```rust
use expand_range::{fill, Value, Step, Options};

let mut opts = Options::new();
opts.to_regex = true;
let r = fill(Value::from(2), Some(Value::from(100)), Step::None, opts);
assert_eq!(r.regex(), "[2-9]|[1-9][0-9]|100");
```

## Behavior

- Direction comes from the bounds. `fill(5, 1)` descends.
- The step magnitude is used. A step of `-2` and `2` behave the same.
- A leading zero on any bound or the step pads every output to a common width.
- String bounds keep their text form, so output stays string typed.
- Bounds may be numbers or single characters. Letter ranges walk character
  codes, so `'a'..'C'` passes through ASCII punctuation.
- Invalid input returns an empty list. Set `strict_ranges` to get a typed error
  through `fill_checked` instead.

## License

Licensed under the [MIT license](LICENSE).
