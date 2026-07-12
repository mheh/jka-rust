//! C `printf`-subset formatter for the variadic seam of `va` / `Com_sprintf`.
//!
//! Shared-tier home for the engine island's `va` (`q_string::va`); a verbatim
//! module-island copy of `mp_game`'s `c_format.rs` (porting-rules §20 —
//! duplicate, don't unify).
//!
//! Rust cannot express C varargs on stable, so the native jampgame DLL's
//! `va`/`Com_sprintf` (which in Raven call the C library's `vsprintf`) take an
//! explicit `&[FmtArg]` argument channel and format through [`c_vsprintf`] here.
//! The target is **native libc `printf` parity** — Raven's DLL build links the
//! real `vsprintf`, not the QVM bytecode fallback in
//! `oracle/codemp/game/bg_lib.c`. That bytecode `vsprintf` is only the
//! reference for *which* directives game code relies on (`%d %i %u %o %x %X %c
//! %s %f %%` with `-`/`0`/`+`/` `/`#` flags, width and precision); its float
//! digit-truncation and unsigned fallbacks are not reproduced here because the
//! shipped DLL never used them.
//! Source: `oracle/codemp/game/bg_lib.c:1183-1288` (directive set);
//! `oracle/codemp/game/q_shared.c:985-1032` (the `vsprintf` callers).

use core::ffi::{c_char, c_int, c_uint};
use std::ffi::CStr;

/// One positional argument for [`c_vsprintf`], typed to the C promotion it
/// stands in for (`int` for `%d %i %c`, `unsigned` for `%u %o %x %X`, `double`
/// for `%f`, `char *` for `%s`).
///
/// Divergence: C reads the argument's type from the directive; the port carries
/// the type on the value so the seam is safe Rust. Callers pass the value they
/// would have pushed on the C stack.
pub enum FmtArg<'a> {
    /// `int` argument for `%d`, `%i`; `%c` uses its low byte.
    Int(c_int),
    /// `unsigned int` argument for `%u`, `%o`, `%x`, `%X`.
    UInt(c_uint),
    /// `double` argument for `%f`.
    Float(f64),
    /// `char *` argument for `%s`. `None` reproduces glibc's `(null)`.
    Str(Option<&'a [u8]>),
}

impl<'a> FmtArg<'a> {
    /// Build a `%s` argument from a NUL-terminated C string pointer, matching
    /// glibc's `(null)` when the pointer is null. The borrow is bounded by the
    /// caller-supplied lifetime; the pointed-to bytes must outlive the format
    /// call (they always do at the `va`/`Com_sprintf` seam — the string is a
    /// live argument).
    ///
    /// # Safety
    /// `p` must be null or point to a valid NUL-terminated string live for `'a`.
    pub unsafe fn cstr(p: *const c_char) -> FmtArg<'a> {
        if p.is_null() {
            FmtArg::Str(None)
        } else {
            FmtArg::Str(Some(CStr::from_ptr(p).to_bytes()))
        }
    }
}

/// Parsed `%`-directive state (`%[flags][width][.prec][length]conv`).
struct Spec {
    minus: bool,
    zero: bool,
    plus: bool,
    space: bool,
    alt: bool,
    width: usize,
    prec: Option<usize>,
    conv: u8,
}

/// Raven's native `vsprintf` (libc) over the directive subset game code uses.
///
/// Formats `fmt` against `args` into a fresh byte buffer and returns it (the
/// caller applies buffer/truncation semantics). Byte-for-byte parity with glibc
/// `printf` for `%d %i %u %o %x %X %c %s %f %%` plus the `-`/`0`/`+`/` `/`#`
/// flags, field width, and precision.
///
/// Any directive outside that subset (`%e %g %p %n`, `*` width/precision, …)
/// panics naming the directive rather than echoing it — the survey of live
/// `va`/`Com_sprintf` format strings finds none, and a silent echo would hide a
/// real mismatch. Argument shortfall/type-mismatch likewise panics.
/// Source: `oracle/codemp/game/bg_lib.c:1183-1288`.
pub fn c_vsprintf(fmt: &[u8], args: &[FmtArg]) -> Vec<u8> {
    let mut out = Vec::with_capacity(fmt.len() + 16);
    let mut ai = 0usize;
    let mut i = 0usize;
    while i < fmt.len() {
        let c = fmt[i];
        if c != b'%' {
            out.push(c);
            i += 1;
            continue;
        }
        // At '%': parse the directive.
        i += 1;
        if i < fmt.len() && fmt[i] == b'%' {
            out.push(b'%');
            i += 1;
            continue;
        }
        let spec = parse_spec(fmt, &mut i);
        let arg = |ai: &mut usize| -> &FmtArg {
            let a = args.get(*ai).unwrap_or_else(|| {
                panic!(
                    "c_vsprintf: missing argument #{} for %{}",
                    *ai + 1,
                    spec.conv as char
                )
            });
            *ai += 1;
            a
        };
        match spec.conv {
            b'd' | b'i' => {
                let v = match arg(&mut ai) {
                    FmtArg::Int(x) => *x as i64,
                    other => type_mismatch("%d/%i", "Int", other),
                };
                let neg = v < 0;
                emit_int(&mut out, v.unsigned_abs(), neg, true, 10, false, &spec);
            }
            b'u' => {
                let v = match arg(&mut ai) {
                    FmtArg::UInt(x) => *x as u64,
                    other => type_mismatch("%u", "UInt", other),
                };
                emit_int(&mut out, v, false, false, 10, false, &spec);
            }
            b'o' => {
                let v = match arg(&mut ai) {
                    FmtArg::UInt(x) => *x as u64,
                    other => type_mismatch("%o", "UInt", other),
                };
                emit_int(&mut out, v, false, false, 8, false, &spec);
            }
            b'x' => {
                let v = match arg(&mut ai) {
                    FmtArg::UInt(x) => *x as u64,
                    other => type_mismatch("%x", "UInt", other),
                };
                emit_int(&mut out, v, false, false, 16, false, &spec);
            }
            b'X' => {
                let v = match arg(&mut ai) {
                    FmtArg::UInt(x) => *x as u64,
                    other => type_mismatch("%X", "UInt", other),
                };
                emit_int(&mut out, v, false, false, 16, true, &spec);
            }
            b'c' => {
                let v = match arg(&mut ai) {
                    FmtArg::Int(x) => *x as u8,
                    other => type_mismatch("%c", "Int", other),
                };
                emit_bytes(&mut out, &[v], &spec);
            }
            b's' => {
                let bytes: &[u8] = match arg(&mut ai) {
                    FmtArg::Str(Some(b)) => b,
                    FmtArg::Str(None) => b"(null)",
                    other => type_mismatch("%s", "Str", other),
                };
                let slice = match spec.prec {
                    Some(p) if p < bytes.len() => &bytes[..p],
                    _ => bytes,
                };
                emit_bytes(&mut out, slice, &spec);
            }
            b'f' | b'F' => {
                let v = match arg(&mut ai) {
                    FmtArg::Float(x) => *x,
                    other => type_mismatch("%f", "Float", other),
                };
                emit_float(&mut out, v, &spec);
            }
            other => panic!(
                "c_vsprintf: unsupported directive %{} — not in the va/Com_sprintf survey",
                other as char
            ),
        }
    }
    out
}

/// Parse `[flags][width][.prec][length]conv` after the leading `%`. `i` points
/// just past the `%`; on return it points just past the conversion char.
fn parse_spec(fmt: &[u8], i: &mut usize) -> Spec {
    let mut s = Spec {
        minus: false,
        zero: false,
        plus: false,
        space: false,
        alt: false,
        width: 0,
        prec: None,
        conv: 0,
    };
    // Flags (order-independent, repeatable).
    while *i < fmt.len() {
        match fmt[*i] {
            b'-' => s.minus = true,
            b'0' => s.zero = true,
            b'+' => s.plus = true,
            b' ' => s.space = true,
            b'#' => s.alt = true,
            _ => break,
        }
        *i += 1;
    }
    // Width.
    if *i < fmt.len() && fmt[*i] == b'*' {
        panic!("c_vsprintf: '*' width unsupported (not in the va/Com_sprintf survey)");
    }
    while *i < fmt.len() && fmt[*i].is_ascii_digit() {
        s.width = s.width * 10 + (fmt[*i] - b'0') as usize;
        *i += 1;
    }
    // Precision.
    if *i < fmt.len() && fmt[*i] == b'.' {
        *i += 1;
        if *i < fmt.len() && fmt[*i] == b'*' {
            panic!("c_vsprintf: '*' precision unsupported (not in the va/Com_sprintf survey)");
        }
        let mut p = 0usize;
        while *i < fmt.len() && fmt[*i].is_ascii_digit() {
            p = p * 10 + (fmt[*i] - b'0') as usize;
            *i += 1;
        }
        s.prec = Some(p);
    }
    // Length modifiers (`l`, `h`, `L`, `z`, `j`, `t`, `q`) — the typed FmtArg
    // already carries the value width, so these are parsed and ignored.
    while *i < fmt.len() && matches!(fmt[*i], b'l' | b'h' | b'L' | b'z' | b'j' | b't' | b'q') {
        *i += 1;
    }
    if *i >= fmt.len() {
        panic!("c_vsprintf: truncated directive (trailing '%')");
    }
    s.conv = fmt[*i];
    *i += 1;
    s
}

/// Emit an integer magnitude `mag` (already sign-stripped) in `base`, applying
/// sign/prefix, precision (minimum digit count), zero/space padding and width.
fn emit_int(
    out: &mut Vec<u8>,
    mut mag: u64,
    neg: bool,
    signed: bool,
    base: u64,
    upper: bool,
    spec: &Spec,
) {
    let digits: &[u8] = if upper {
        b"0123456789ABCDEF"
    } else {
        b"0123456789abcdef"
    };
    let mut num: Vec<u8> = Vec::new();
    if mag == 0 {
        // C: precision 0 with a zero value produces no digits.
        if spec.prec != Some(0) {
            num.push(b'0');
        }
    } else {
        while mag > 0 {
            num.push(digits[(mag % base) as usize]);
            mag /= base;
        }
        num.reverse();
    }
    // Precision = minimum number of digits (leading zeros).
    if let Some(p) = spec.prec {
        while num.len() < p {
            num.insert(0, b'0');
        }
    }
    // Sign: `-` only for negatives; `+`/` ` apply to signed conversions only
    // (glibc ignores them for %u/%o/%x/%X).
    let sign: &[u8] = if neg {
        b"-"
    } else if signed && spec.plus {
        b"+"
    } else if signed && spec.space {
        b" "
    } else {
        b""
    };
    // `#` prefix: `0x`/`0X` for nonzero hex; leading `0` for octal.
    let mut prefix: Vec<u8> = Vec::new();
    if spec.alt {
        if base == 16 && !num.is_empty() && !(num.len() == 1 && num[0] == b'0') {
            prefix.extend_from_slice(if upper { b"0X" } else { b"0x" });
        } else if base == 8 && (num.is_empty() || num[0] != b'0') {
            prefix.push(b'0');
        }
    }
    let body_len = sign.len() + prefix.len() + num.len();
    // Zero-pad applies only when not left-adjusted and no explicit precision.
    let zero_pad = spec.zero && !spec.minus && spec.prec.is_none();
    if spec.width > body_len {
        let pad = spec.width - body_len;
        if spec.minus {
            out.extend_from_slice(sign);
            out.extend_from_slice(&prefix);
            out.extend_from_slice(&num);
            out.extend(std::iter::repeat(b' ').take(pad));
        } else if zero_pad {
            out.extend_from_slice(sign);
            out.extend_from_slice(&prefix);
            out.extend(std::iter::repeat(b'0').take(pad));
            out.extend_from_slice(&num);
        } else {
            out.extend(std::iter::repeat(b' ').take(pad));
            out.extend_from_slice(sign);
            out.extend_from_slice(&prefix);
            out.extend_from_slice(&num);
        }
    } else {
        out.extend_from_slice(sign);
        out.extend_from_slice(&prefix);
        out.extend_from_slice(&num);
    }
}

/// Emit raw bytes (`%c` / `%s` body, already precision-truncated) with width
/// padding. `0` flag has no effect on string/char conversions in glibc.
fn emit_bytes(out: &mut Vec<u8>, body: &[u8], spec: &Spec) {
    if spec.width > body.len() {
        let pad = spec.width - body.len();
        if spec.minus {
            out.extend_from_slice(body);
            out.extend(std::iter::repeat(b' ').take(pad));
        } else {
            out.extend(std::iter::repeat(b' ').take(pad));
            out.extend_from_slice(body);
        }
    } else {
        out.extend_from_slice(body);
    }
}

/// Emit a `%f` value. Magnitude is rendered by Rust's fixed-precision formatter
/// (correctly rounded, round-half-to-even — the same default as glibc); sign,
/// zero/space padding and width are applied here.
fn emit_float(out: &mut Vec<u8>, v: f64, spec: &Spec) {
    let prec = spec.prec.unwrap_or(6);
    let neg = v.is_sign_negative() && !v.is_nan();
    let body: Vec<u8> = if v.is_nan() {
        b"nan".to_vec()
    } else if v.is_infinite() {
        b"inf".to_vec()
    } else {
        format!("{:.*}", prec, v.abs()).into_bytes()
    };
    let sign: &[u8] = if neg {
        b"-"
    } else if spec.plus {
        b"+"
    } else if spec.space {
        b" "
    } else {
        b""
    };
    let body_len = sign.len() + body.len();
    // Zero-pad numeric floats (never inf/nan) when not left-adjusted.
    let zero_pad = spec.zero && !spec.minus && v.is_finite();
    if spec.width > body_len {
        let pad = spec.width - body_len;
        if spec.minus {
            out.extend_from_slice(sign);
            out.extend_from_slice(&body);
            out.extend(std::iter::repeat(b' ').take(pad));
        } else if zero_pad {
            out.extend_from_slice(sign);
            out.extend(std::iter::repeat(b'0').take(pad));
            out.extend_from_slice(&body);
        } else {
            out.extend(std::iter::repeat(b' ').take(pad));
            out.extend_from_slice(sign);
            out.extend_from_slice(&body);
        }
    } else {
        out.extend_from_slice(sign);
        out.extend_from_slice(&body);
    }
}

fn type_mismatch(conv: &str, expected: &str, got: &FmtArg) -> ! {
    let got_name = match got {
        FmtArg::Int(_) => "Int",
        FmtArg::UInt(_) => "UInt",
        FmtArg::Float(_) => "Float",
        FmtArg::Str(_) => "Str",
    };
    panic!("c_vsprintf: {conv} needs FmtArg::{expected}, got FmtArg::{got_name}");
}

#[cfg(test)]
mod tests {
    use super::{c_vsprintf, FmtArg};

    // Every expected string below is exactly what glibc `printf` produces for
    // the same format+args (cross-checked against a throwaway C program).
    fn f(fmt: &str, args: &[FmtArg]) -> String {
        String::from_utf8(c_vsprintf(fmt.as_bytes(), args)).unwrap()
    }

    #[test]
    fn literal_and_percent() {
        assert_eq!(f("plain text", &[]), "plain text");
        assert_eq!(f("100%% done", &[]), "100% done");
        assert_eq!(f("%%%%", &[]), "%%");
    }

    #[test]
    fn signed_int_d_i() {
        assert_eq!(f("%d", &[FmtArg::Int(0)]), "0");
        assert_eq!(f("%d", &[FmtArg::Int(42)]), "42");
        assert_eq!(f("%i", &[FmtArg::Int(-42)]), "-42");
        assert_eq!(f("%d", &[FmtArg::Int(2147483647)]), "2147483647");
        assert_eq!(f("%d", &[FmtArg::Int(-2147483648)]), "-2147483648");
    }

    #[test]
    fn int_width_and_zeropad() {
        assert_eq!(f("%5d", &[FmtArg::Int(42)]), "   42");
        assert_eq!(f("%05d", &[FmtArg::Int(42)]), "00042");
        assert_eq!(f("%05d", &[FmtArg::Int(-42)]), "-0042");
        assert_eq!(f("%-5d", &[FmtArg::Int(42)]), "42   ");
        assert_eq!(f("%-5d|", &[FmtArg::Int(-42)]), "-42  |");
        assert_eq!(f("%3d", &[FmtArg::Int(12345)]), "12345");
        assert_eq!(f("%4i", &[FmtArg::Int(7)]), "   7");
    }

    #[test]
    fn int_sign_flags() {
        assert_eq!(f("%+d", &[FmtArg::Int(42)]), "+42");
        assert_eq!(f("%+d", &[FmtArg::Int(-42)]), "-42");
        assert_eq!(f("% d", &[FmtArg::Int(42)]), " 42");
        assert_eq!(f("% d", &[FmtArg::Int(-42)]), "-42");
        assert_eq!(f("%+05d", &[FmtArg::Int(42)]), "+0042");
    }

    #[test]
    fn int_precision() {
        assert_eq!(f("%.5d", &[FmtArg::Int(42)]), "00042");
        assert_eq!(f("%.0d", &[FmtArg::Int(0)]), "");
        assert_eq!(f("%.3d", &[FmtArg::Int(0)]), "000");
        assert_eq!(f("%8.5d", &[FmtArg::Int(42)]), "   00042");
        // Precision disables the zero flag.
        assert_eq!(f("%08.5d", &[FmtArg::Int(42)]), "   00042");
    }

    #[test]
    fn unsigned_hex_octal() {
        assert_eq!(f("%u", &[FmtArg::UInt(4294967295)]), "4294967295");
        assert_eq!(f("%x", &[FmtArg::UInt(255)]), "ff");
        assert_eq!(f("%X", &[FmtArg::UInt(255)]), "FF");
        assert_eq!(f("%08x", &[FmtArg::UInt(255)]), "000000ff");
        assert_eq!(f("%#x", &[FmtArg::UInt(255)]), "0xff");
        assert_eq!(f("%#X", &[FmtArg::UInt(255)]), "0XFF");
        assert_eq!(f("%#x", &[FmtArg::UInt(0)]), "0");
        assert_eq!(f("%o", &[FmtArg::UInt(8)]), "10");
        assert_eq!(f("%#o", &[FmtArg::UInt(8)]), "010");
        // Space flag is ignored for unsigned octal (matches glibc).
        assert_eq!(f("% o", &[FmtArg::UInt(8)]), "10");
    }

    #[test]
    fn char_conv() {
        assert_eq!(f("%c", &[FmtArg::Int(b'A' as i32)]), "A");
        assert_eq!(f("[%3c]", &[FmtArg::Int(b'A' as i32)]), "[  A]");
        assert_eq!(f("[%-3c]", &[FmtArg::Int(b'A' as i32)]), "[A  ]");
        // Space flag is a no-op for %c.
        assert_eq!(f("% c", &[FmtArg::Int(b'A' as i32)]), "A");
    }

    #[test]
    fn string_conv() {
        let s: &[u8] = b"hello";
        assert_eq!(f("%s", &[FmtArg::Str(Some(s))]), "hello");
        assert_eq!(f("%8s", &[FmtArg::Str(Some(s))]), "   hello");
        assert_eq!(f("%-8s|", &[FmtArg::Str(Some(s))]), "hello   |");
        assert_eq!(f("%.3s", &[FmtArg::Str(Some(s))]), "hel");
        assert_eq!(f("%8.3s", &[FmtArg::Str(Some(s))]), "     hel");
        assert_eq!(
            f("%-20s|", &[FmtArg::Str(Some(s))]),
            "hello               |"
        );
        assert_eq!(f("%s", &[FmtArg::Str(None)]), "(null)");
    }

    #[test]
    fn float_default_and_precision() {
        assert_eq!(f("%f", &[FmtArg::Float(3.14159)]), "3.141590");
        assert_eq!(f("%f", &[FmtArg::Float(0.0)]), "0.000000");
        assert_eq!(f("%.2f", &[FmtArg::Float(3.14159)]), "3.14");
        assert_eq!(f("%.0f", &[FmtArg::Float(3.7)]), "4");
        assert_eq!(f("%.0f", &[FmtArg::Float(2.5)]), "2"); // round-half-to-even
        assert_eq!(f("%.0f", &[FmtArg::Float(3.5)]), "4"); // round-half-to-even
        assert_eq!(f("%.1f", &[FmtArg::Float(0.05)]), "0.1");
        assert_eq!(f("%.2f", &[FmtArg::Float(-3.14159)]), "-3.14");
        assert_eq!(f("%1.8f", &[FmtArg::Float(0.5)]), "0.50000000");
        assert_eq!(f("%.2f", &[FmtArg::Float(2.0)]), "2.00");
    }

    #[test]
    fn float_width_and_flags() {
        assert_eq!(f("%8.2f", &[FmtArg::Float(3.14159)]), "    3.14");
        assert_eq!(f("%08.2f", &[FmtArg::Float(3.14159)]), "00003.14");
        assert_eq!(f("%08.2f", &[FmtArg::Float(-3.14159)]), "-0003.14");
        assert_eq!(f("%-8.2f|", &[FmtArg::Float(3.14159)]), "3.14    |");
        assert_eq!(f("%+.2f", &[FmtArg::Float(3.14159)]), "+3.14");
        assert_eq!(f("% .2f", &[FmtArg::Float(3.14159)]), " 3.14");
        assert_eq!(f("%5.2f", &[FmtArg::Float(3.14159)]), " 3.14");
        assert_eq!(f("%4.2f", &[FmtArg::Float(3.14159)]), "3.14");
        assert_eq!(f("%0.0f", &[FmtArg::Float(42.9)]), "43");
    }

    #[test]
    fn negative_zero_float() {
        assert_eq!(f("%.2f", &[FmtArg::Float(-0.0)]), "-0.00");
    }

    #[test]
    fn multi_arg_mixed() {
        let name: &[u8] = b"Kyle";
        assert_eq!(
            f(
                "%s has %d frags (%.1f%% acc)",
                &[
                    FmtArg::Str(Some(name)),
                    FmtArg::Int(15),
                    FmtArg::Float(42.5)
                ]
            ),
            "Kyle has 15 frags (42.5% acc)"
        );
        // Mirrors the g_client PLCONNECT/name-change broadcasts.
        let old: &[u8] = b"OldName";
        let new: &[u8] = b"NewName";
        assert_eq!(
            f(
                "print \"%s %s %s\n\"",
                &[
                    FmtArg::Str(Some(old)),
                    FmtArg::Str(Some(b"renamed to")),
                    FmtArg::Str(Some(new))
                ]
            ),
            "print \"OldName renamed to NewName\n\""
        );
    }

    #[test]
    #[should_panic(expected = "unsupported directive %e")]
    fn unsupported_directive_panics() {
        c_vsprintf(b"%e", &[FmtArg::Float(1.0)]);
    }

    #[test]
    #[should_panic(expected = "missing argument")]
    fn missing_arg_panics() {
        c_vsprintf(b"%d", &[]);
    }
}
