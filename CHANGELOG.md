# Changelog

## [0.2.0] - 2026-07-07

### Changed
- Numeric ranges near `i64::MAX` now return exact endpoint values instead of losing precision. (#18)
- Regex output for integer strings such as `+2` and `1.` now emits the coerced integer value, so the pattern compiles and matches those numbers. (#19)
- Character regex ranges with `-` as an endpoint now escape the hyphen and match the full requested interval. (#22)
- Regex output for negative padded ranges now uses the bound width and no longer accepts extra leading zeros in strict zero mode. (#23)

### Fixed
- Letter ranges with a step that would overflow now stop cleanly instead of panicking. (#20)
- Regex output for stepped ranges that include `i64::MIN` now returns a valid negative pattern. (#21)

## [0.2.0] - 2026-07-07

### Changed
- Numeric ranges near `i64::MAX` now return exact endpoint values instead of losing precision. (#18)
- Regex output for integer strings such as `+2` and `1.` now emits the coerced integer value, so the pattern compiles and matches those numbers. (#19)
- Character regex ranges with `-` as an endpoint now escape the hyphen and match the full requested interval. (#22)
- Regex output for negative padded ranges now uses the bound width and no longer accepts extra leading zeros in strict zero mode. (#23)

### Fixed
- Letter ranges with a step that would overflow now stop cleanly instead of panicking. (#20)
- Regex output for stepped ranges that include `i64::MIN` now returns a valid negative pattern. (#21)
