//! `atof()` matching the **native libc** `atof` linked by the oracle DLL
//! (`atof(s)` is `strtod(s, NULL)`) — not `bg_lib`'s QVM-only bytecode port
//! (`oracle/codemp/game/bg_lib.c`), never compiled into the native build.

use crate::ctype::isspace_byte;

/// libc `atof` is `strtod(s, NULL)`: skip libc `isspace` whitespace, optional
/// single sign, then the longest valid decimal prefix — digits with an
/// optional fraction (`.` + digits) and an optional exponent (`e`/`E`,
/// optional sign, at least one digit or the exponent is not consumed:
/// `"1e+"` -> 1.0) — or a case-insensitive `inf`/`infinity`/`nan` form. No
/// valid prefix parses as 0.0. Decimal conversion is correctly rounded (both
/// strtod and Rust's `f64` parser round to nearest-even). Hex-float prefixes
/// are not recognized (no oracle string feeds them); they stop at the `x`.
pub fn atof(text: &str) -> f64 {
    atof_bytes(text.as_bytes())
}

/// [`atof`] over raw bytes, for seam callers that hold unconverted C data.
pub fn atof_bytes(bytes: &[u8]) -> f64 {
    let mut i = 0usize;
    while i < bytes.len() && isspace_byte(bytes[i]) {
        i += 1;
    }
    let neg = match bytes.get(i) {
        Some(b'+') => {
            i += 1;
            false
        }
        Some(b'-') => {
            i += 1;
            true
        }
        _ => false,
    };

    let rest = &bytes[i..];
    if starts_with_ignore_case(rest, b"infinity") || starts_with_ignore_case(rest, b"inf") {
        return if neg { f64::NEG_INFINITY } else { f64::INFINITY };
    }
    if starts_with_ignore_case(rest, b"nan") {
        // strtod sets the NaN sign bit for "-nan"; no caller distinguishes it.
        return f64::NAN;
    }

    // Longest decimal prefix: digits [. digits] [(e|E) [sign] digits].
    let start = i;
    let mut end = i;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    let int_digits = end - start;
    let mut frac_digits = 0;
    if bytes.get(end) == Some(&b'.') {
        let mut f = end + 1;
        while f < bytes.len() && bytes[f].is_ascii_digit() {
            f += 1;
        }
        frac_digits = f - (end + 1);
        // A lone "." with no digits on either side is not a conversion.
        if int_digits + frac_digits > 0 {
            end = f;
        }
    }
    if int_digits + frac_digits == 0 {
        return 0.0;
    }
    if let Some(&e) = bytes.get(end) {
        if e == b'e' || e == b'E' {
            let mut x = end + 1;
            if matches!(bytes.get(x), Some(b'+') | Some(b'-')) {
                x += 1;
            }
            let first_exp_digit = x;
            while x < bytes.len() && bytes[x].is_ascii_digit() {
                x += 1;
            }
            if x > first_exp_digit {
                end = x;
            }
        }
    }

    // The prefix is all-ASCII (digits/./e/sign), so it is valid UTF-8 and
    // Rust's f64 parser accepts exactly this grammar.
    let magnitude: f64 = core::str::from_utf8(&bytes[start..end])
        .unwrap()
        .parse()
        .unwrap();
    if neg {
        -magnitude
    } else {
        magnitude
    }
}

fn starts_with_ignore_case(bytes: &[u8], prefix: &[u8]) -> bool {
    bytes.len() >= prefix.len() && bytes[..prefix.len()].eq_ignore_ascii_case(prefix)
}

#[cfg(test)]
mod atof_tests {
    use super::atof;

    #[test]
    fn plain_and_signed_decimals() {
        assert_eq!(atof("3.25"), 3.25);
        assert_eq!(atof("+7"), 7.0);
        assert_eq!(atof(" \t-42"), -42.0);
    }

    #[test]
    fn exponent_and_trailing_garbage() {
        assert_eq!(atof("  -2.5e3xyz"), -2500.0);
        assert_eq!(atof("1.5x"), 1.5);
    }

    #[test]
    fn empty_and_non_numeric() {
        assert_eq!(atof(""), 0.0);
        assert_eq!(atof("abc"), 0.0);
        assert_eq!(atof("."), 0.0);
    }

    #[test]
    fn bare_fraction_and_bare_point() {
        assert_eq!(atof(".5"), 0.5);
        assert_eq!(atof("5."), 5.0);
    }

    #[test]
    fn incomplete_exponent_is_not_consumed() {
        assert_eq!(atof("1e"), 1.0);
        assert_eq!(atof("1e+"), 1.0);
    }

    #[test]
    fn vertical_tab_is_libc_isspace() {
        assert_eq!(atof("\x0b42"), 42.0);
    }

    #[test]
    fn infinity_and_nan_forms() {
        assert_eq!(atof("inf"), f64::INFINITY);
        assert_eq!(atof("-INFINITY"), f64::NEG_INFINITY);
        assert!(atof("nan").is_nan());
    }

    #[test]
    fn hex_prefix_stops_at_x() {
        assert_eq!(atof("0x10"), 0.0);
    }
}
