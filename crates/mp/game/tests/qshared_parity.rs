//! Differential parity test for the jampgame `q_shared` port against the Raven
//! oracle. Reproduces `tests/oracle/golden/qshared.txt` (generated
//! only by the C dumper `main_qshared.c` over the committed
//! `fixtures/qshared/`) by calling the PORTED `mp_game::q_shared` functions and
//! byte-comparing to the golden.
//!
//! Single-threaded, single process by construction: the oracle keeps parser
//! state in file statics (`com_lines`, `va`'s rotating buffer), and the port
//! mirrors that with module-level `static mut`s. The whole dump is ONE `#[test]`
//! whose sections run in the exact order the C dumper emits them, so the shared
//! statics evolve identically on both sides. Run with `--test-threads=1`.
//! See `tools/jampgame-oracle/README.md`.
//!
//! Scope note: `Com_sprintf`/`va` are exercised here with LITERAL formats only
//! (the golden fixture predates real `%`-substitution). The `%`-directive
//! substitution itself now lives in `mp_game::c_format::c_vsprintf` and is
//! covered byte-exactly by that module's own `#[cfg(test)]` suite; the literal
//! cases below still validate the rotating-buffer and truncation semantics.
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};
use std::fmt::Write as _;

use mp_game::prelude::cstr;
use mp_game::q_shared::{
    va, COM_BeginParseSession, COM_Compress, COM_DefaultExtension, COM_GetCurrentParseLine,
    COM_ParseExt, COM_StripExtension, Com_sprintf, Info_SetValueForKey, Info_SetValueForKey_Big,
    QSharedScratch, Q_CleanStr, Q_PrintStrlen, Q_isalpha, Q_islower, Q_isprint, Q_isupper,
    Q_strcat, Q_stricmp, Q_stricmpn, Q_strlwr, Q_strncmp, Q_strncpyz, Q_strrchr, Q_strupr,
    SkipBracedSection, SkipRestOfLine,
};
use native_math::qmath::{Com_Clamp, Com_Clampi};
use native_string::info::{Info_RemoveKey, Info_RemoveKey_Big, Info_Validate, Info_ValueForKey};
use testkit::{compare, oracle_dir};

fn read_fixture(name: &str) -> Vec<u8> {
    let path = oracle_dir(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/qshared")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// --- canonical emit helpers (mirror main_qshared.c byte-for-byte) ---

/// Read a C string (up to NUL) into a byte vector.
fn read_cstr(p: *const c_char) -> Vec<u8> {
    let mut v = Vec::new();
    let mut q = p;
    unsafe {
        while *q != 0 {
            v.push(*q as u8);
            q = q.add(1);
        }
    }
    v
}

/// A quoted, escaped string: printable ASCII except `"`/`\` verbatim, else \xHH.
fn qstr(o: &mut String, bytes: &[u8]) {
    o.push('"');
    for &c in bytes {
        if (0x20..=0x7e).contains(&c) && c != b'"' && c != b'\\' {
            o.push(c as char);
        } else {
            let _ = write!(o, "\\x{c:02x}");
        }
    }
    o.push('"');
}

fn qstr_p(o: &mut String, p: *const c_char) {
    let b = read_cstr(p);
    qstr(o, &b);
}

/// A fixed byte window as space-separated %02x.
fn hex(o: &mut String, b: &[c_char]) {
    for (i, &c) in b.iter().enumerate() {
        let _ = write!(o, "{}{:02x}", if i > 0 { " " } else { "" }, c as u8);
    }
}

/// Build a NUL-terminated `Vec<c_char>` from bytes.
fn cbuf_b(b: &[u8]) -> Vec<c_char> {
    let mut v: Vec<c_char> = b.iter().map(|&x| x as c_char).collect();
    v.push(0);
    v
}
fn cbuf(s: &str) -> Vec<c_char> {
    cbuf_b(s.as_bytes())
}

const NULLP: *const c_char = core::ptr::null();

// ============================ sections ============================

fn dump_clamp(o: &mut String) {
    o.push_str("== clamp ==\n");
    let iv = [-100i32, -1, 0, 1, 5, 50, 100];
    for &a in &iv {
        for &b in &iv {
            for &c in &iv {
                let _ = writeln!(o, "ci {}", Com_Clampi(a, b, c));
            }
        }
    }
    let cv = [-100.0f32, -1.5, 0.0, 1.5, 5.25, 100.0];
    for &a in &cv {
        for &b in &cv {
            for &c in &cv {
                let _ = writeln!(o, "cf {:08x}", Com_Clamp(a, b, c).to_bits());
            }
        }
    }
}

// Feed the tokenizer a NUL-terminated byte buffer — the byte cursor treats an
// embedded NUL as the C-string end (§19), reproducing the pointer version that
// walked a `char*` past its terminator.
fn nul_terminated(bytes: &[u8]) -> Vec<u8> {
    let mut v = bytes.to_vec();
    v.push(0);
    v
}

fn dump_tokens(o: &mut String) {
    o.push_str("== tokens ==\n");
    let cb = nul_terminated(&read_fixture("tokens.txt"));
    let mut qs = QSharedScratch::zeroed();

    // pass 1: allowLineBreaks = qtrue
    let name = "tokens";
    COM_BeginParseSession(&mut qs, name);
    let mut p: Option<&[u8]> = Some(&cb);
    for _ in 0..200 {
        let (tok, rest) = COM_ParseExt(&mut qs, p, true);
        p = rest;
        let eof = p.is_none();
        let _ = write!(o, "qt ");
        qstr(o, tok.as_bytes());
        let _ = writeln!(
            o,
            " line {} nul {}",
            COM_GetCurrentParseLine(&qs),
            eof as i32
        );
        if tok.is_empty() && eof {
            break;
        }
    }

    // pass 2: allowLineBreaks = qfalse (empty token returned at line breaks)
    COM_BeginParseSession(&mut qs, name);
    p = Some(&cb);
    for _ in 0..200 {
        let (tok, rest) = COM_ParseExt(&mut qs, p, false);
        p = rest;
        let eof = p.is_none();
        let _ = write!(o, "qf ");
        qstr(o, tok.as_bytes());
        let _ = writeln!(
            o,
            " line {} nul {}",
            COM_GetCurrentParseLine(&qs),
            eof as i32
        );
        if eof {
            break;
        }
    }
}

fn dump_compress(o: &mut String) {
    o.push_str("== compress ==\n");
    let data = read_fixture("compress.txt");
    for (idx, rec) in data.split(|&b| b == 0).enumerate() {
        let mut cb = cbuf_b(rec);
        let r = COM_Compress(cb.as_mut_ptr());
        let _ = write!(o, "cz {idx} len {r} ");
        qstr_p(o, cb.as_ptr());
        let _ = writeln!(o);
    }
}

fn dump_braced(o: &mut String) {
    o.push_str("== braced ==\n");
    let cases = [
        "{ a { b } c } after",
        "{ } trailing",
        "{nested{deeper{x}}} end",
        "{ unterminated { block ",
        "noBrace token here",
    ];
    let name = "braced";
    let mut qs = QSharedScratch::zeroed();
    for (i, input) in cases.iter().enumerate() {
        let cb = nul_terminated(input.as_bytes());
        COM_BeginParseSession(&mut qs, name);
        let p: Option<&[u8]> = Some(&cb);
        let p = SkipBracedSection(&mut qs, p);
        let off: i64 = match p {
            None => -1,
            Some(rest) => (cb.len() - rest.len()) as i64,
        };
        let rest_tok: String = match p {
            None => String::new(),
            Some(_) => COM_ParseExt(&mut qs, p, true).0,
        };
        let line = COM_GetCurrentParseLine(&qs);
        let _ = write!(o, "br {i} off {off} line {line} rest ");
        qstr(o, rest_tok.as_bytes());
        let _ = writeln!(o);
    }
}

fn dump_skipline(o: &mut String) {
    o.push_str("== skipline ==\n");
    let cases = [
        "rest of line\nnext line",
        "no newline here",
        "\nimmediate",
        "a\nb\nc",
    ];
    let name = "skipline";
    let mut qs = QSharedScratch::zeroed();
    for (i, input) in cases.iter().enumerate() {
        let cb = nul_terminated(input.as_bytes());
        COM_BeginParseSession(&mut qs, name);
        let p: Option<&[u8]> = Some(&cb);
        let p = SkipRestOfLine(&mut qs, p);
        // Raven's SkipRestOfLine consumes the terminating NUL when there is no
        // newline, leaving the cursor one past the NUL; the byte cursor
        // reproduces that (the NUL-terminated buffer's `\0` is consumed), so the
        // offset matches the pointer version. Only offset + line are dumped (§19).
        let off: i64 = match p {
            None => -1,
            Some(rest) => (cb.len() - rest.len()) as i64,
        };
        let line = COM_GetCurrentParseLine(&qs);
        let _ = writeln!(o, "sl {i} off {off} line {line}");
    }
}

fn dump_strhelpers(o: &mut String) {
    o.push_str("== strhelpers ==\n");

    // isXXX predicates over signed-char edges.
    let ic = [
        -1i32, 0, 0x1f, 0x20, 0x40, 0x5a, 0x61, 0x7a, 0x7e, 0x7f, 0x80, 0xff,
    ];
    for &c in &ic {
        let _ = writeln!(
            o,
            "is {} p {} l {} u {} a {}",
            c,
            Q_isprint(c),
            Q_islower(c),
            Q_isupper(c),
            Q_isalpha(c)
        );
    }

    // COM_StripExtension.
    let sx = [
        "",
        "file",
        "file.ext",
        "a.b.c",
        ".hidden",
        "no_dot",
        "path/to/file.tga",
    ];
    for s in sx {
        let inb = cbuf(s);
        let mut out = vec![0 as c_char; 64];
        COM_StripExtension(inb.as_ptr(), out.as_mut_ptr());
        let _ = write!(o, "strip ");
        qstr(o, s.as_bytes());
        let _ = write!(o, " -> ");
        qstr_p(o, out.as_ptr());
        let _ = writeln!(o);
    }

    // COM_DefaultExtension (non-empty paths; empty path reads path[-1] = UB).
    let dx = [
        ("file", ".tga"),
        ("file.bmp", ".tga"),
        ("path/to/file", ".md3"),
        ("path/to/file.ext", ".md3"),
        ("noext", ".cfg"),
        ("dir.x/name", ".wav"),
    ];
    for (p, e) in dx {
        let mut path = vec![0 as c_char; 128];
        for (i, b) in p.bytes().enumerate() {
            path[i] = b as c_char;
        }
        let ext = cbuf(e);
        COM_DefaultExtension(path.as_mut_ptr(), 128, ext.as_ptr());
        let _ = write!(o, "defext ");
        qstr(o, p.as_bytes());
        let _ = write!(o, " ");
        qstr(o, e.as_bytes());
        let _ = write!(o, " -> ");
        qstr_p(o, path.as_ptr());
        let _ = writeln!(o);
    }

    // Q_strncpyz (24-byte window filled with 0xAA to observe zero-padding).
    let ncz = ["", "hi", "exactfit", "longerthanthebuffer.....", "abc"];
    let nsz = [1i32, 2, 4, 8, 16];
    for (i, s) in ncz.iter().enumerate() {
        for &sz in &nsz {
            let mut b = vec![0xAAu8 as c_char; 24];
            let src = cbuf(s);
            Q_strncpyz(b.as_mut_ptr(), src.as_ptr(), sz);
            let _ = write!(o, "ncpyz {i} {sz} ");
            hex(o, &b);
            let _ = writeln!(o);
        }
    }

    // Q_strcat (skip combos the C would Com_Error-abort on: strlen(init) >= size).
    let cat_i = ["", "foo", "12345"];
    let cat_s = ["", "bar", "appendmelong"];
    let cat_sz = [4i32, 8, 16];
    for (a, init) in cat_i.iter().enumerate() {
        for (b, s) in cat_s.iter().enumerate() {
            for &z in &cat_sz {
                if init.len() as c_int >= z {
                    continue;
                }
                let mut buf = vec![0xAAu8 as c_char; 24];
                for (k, by) in init.bytes().enumerate() {
                    buf[k] = by as c_char;
                }
                buf[init.len()] = 0;
                let src = cbuf(s);
                Q_strcat(buf.as_mut_ptr(), z, src.as_ptr());
                let _ = write!(o, "cat {a} {b} {z} ");
                hex(o, &buf);
                let _ = writeln!(o);
            }
        }
    }

    // Q_stricmp / Q_stricmpn / Q_strncmp over case/prefix/high-bit pairs.
    let cmp_a: [&[u8]; 14] = [
        b"",
        b"a",
        b"a",
        b"abc",
        b"abc",
        b"Hello",
        b"hello",
        b"abc",
        b"ab",
        b"zoo",
        b"Test123",
        b"\x80x",
        b"a\x80",
        b"MixedCase",
    ];
    let cmp_b: [&[u8]; 14] = [
        b"",
        b"a",
        b"A",
        b"abd",
        b"abc",
        b"hello",
        b"HELLO",
        b"ab",
        b"abc",
        b"zoon",
        b"test123",
        b"\x80x",
        b"a\x7f",
        b"mixedcase",
    ];
    let cmp_n = [0i32, 1, 2, 3, 5, 99999];
    for i in 0..cmp_a.len() {
        let a = cbuf_b(cmp_a[i]);
        let b = cbuf_b(cmp_b[i]);
        let _ = writeln!(o, "stricmp {i} {}", Q_stricmp(a.as_ptr(), b.as_ptr()));
        for &n in &cmp_n {
            let _ = writeln!(
                o,
                "stricmpn {i} {n} {}",
                Q_stricmpn(a.as_ptr(), b.as_ptr(), n)
            );
        }
        for &n in &cmp_n {
            let _ = writeln!(
                o,
                "strncmp {i} {n} {}",
                Q_strncmp(a.as_ptr(), b.as_ptr(), n)
            );
        }
    }
    let x = cbuf("x");
    let _ = writeln!(
        o,
        "stricmp_null {} {} {}",
        Q_stricmp(NULLP, x.as_ptr()),
        Q_stricmp(x.as_ptr(), NULLP),
        Q_stricmp(NULLP, NULLP)
    );
    let _ = writeln!(
        o,
        "stricmpn_null {} {} {}",
        Q_stricmpn(NULLP, NULLP, 5),
        Q_stricmpn(NULLP, x.as_ptr(), 5),
        Q_stricmpn(x.as_ptr(), NULLP, 5)
    );

    // Q_strlwr / Q_strupr (ASCII only).
    let lu = ["", "Hello World", "ALLCAPS", "already lower", "MiXeD123!@#"];
    for s in lu {
        let mut a = cbuf(s);
        let mut b = cbuf(s);
        Q_strlwr(a.as_mut_ptr());
        Q_strupr(b.as_mut_ptr());
        let _ = write!(o, "lwr ");
        qstr_p(o, a.as_ptr());
        let _ = write!(o, " upr ");
        qstr_p(o, b.as_ptr());
        let _ = writeln!(o);
    }

    // Q_PrintStrlen / Q_CleanStr over color/control-byte strings.
    let col: [&[u8]; 11] = [
        b"",
        b"hello",
        b"^1red",
        b"^1r^2g^3b",
        b"^^literal",
        b"^8notcolor",
        b"trailing^",
        b"^",
        b"a\x01b\x1fc\x7fd\x80e",
        b"^7white^0black^",
        b"plain text 123",
    ];
    for (i, c) in col.iter().enumerate() {
        let sb = cbuf_b(c);
        let _ = writeln!(o, "pslen {i} {}", Q_PrintStrlen(sb.as_ptr()));
        let mut cs = cbuf_b(c);
        Q_CleanStr(cs.as_mut_ptr());
        let _ = write!(o, "clean {i} ");
        qstr_p(o, cs.as_ptr());
        let _ = writeln!(o);
    }
    let _ = writeln!(o, "pslen_null {}", Q_PrintStrlen(NULLP));

    // Q_strrchr: dump offset of the found char (or -1).
    let rc = [
        "",
        "a",
        "hello",
        "abracadabra",
        "a/b/c/d",
        "trailing/",
        "^1color",
    ];
    let rcc = [
        b'a' as c_int,
        b'/' as c_int,
        b'z' as c_int,
        0,
        b'r' as c_int,
    ];
    for (i, s) in rc.iter().enumerate() {
        let cb = cbuf(s);
        for &c in &rcc {
            let r = Q_strrchr(cb.as_ptr(), c);
            let off: i64 = if r.is_null() {
                -1
            } else {
                unsafe { r.offset_from(cb.as_ptr()) as i64 }
            };
            let _ = writeln!(o, "rrchr {i} {c} {off}");
        }
    }
}

fn dump_va(o: &mut String) {
    o.push_str("== va ==\n");
    let mut qs = QSharedScratch::zeroed();
    let f1 = cstr("first-literal");
    let p1 = va(&mut qs, f1.as_ptr(), &[]);
    let f2 = cstr("second-literal");
    let p2 = va(&mut qs, f2.as_ptr(), &[]);
    let _ = write!(o, "va1 ");
    qstr_p(o, p1);
    let _ = writeln!(o);
    let _ = write!(o, "va2 ");
    qstr_p(o, p2);
    let _ = writeln!(o);
    let f3 = cstr("third-literal");
    let p3 = va(&mut qs, f3.as_ptr(), &[]); // reuses p1's slot (2-slot rotation)
    let _ = write!(o, "va1b ");
    qstr_p(o, p1);
    let _ = writeln!(o);
    let _ = write!(o, "va2b ");
    qstr_p(o, p2);
    let _ = writeln!(o);
    let _ = write!(o, "va3 ");
    qstr_p(o, p3);
    let _ = writeln!(o);
}

fn dump_sprintf(o: &mut String) {
    o.push_str("== sprintf ==\n");
    let cases = [
        ("sp1", 24i32, "hello world"),
        ("sp2", 8, "truncate me please"),
        ("sp3", 24, ""),
        ("sp4", 1, "anything"),
    ];
    for (tag, size, lit) in cases {
        let mut b = vec![0xAAu8 as c_char; 24];
        let f = cbuf(lit);
        Com_sprintf(b.as_mut_ptr(), size, f.as_ptr(), &[]);
        let _ = write!(o, "{tag} ");
        hex(o, &b);
        let _ = writeln!(o);
    }
}

// --- info: probe key tables mirror main_qshared.c ---
const VKEYS: &[&str] = &[
    "name", "team", "key2", "empty", "onlykey", "desc", "missing", "Name", "",
];
const RKEYS: &[&str] = &[
    "name", "team", "key1", "desc", "onlykey", "missing", "quote", "semi",
];
const SKV: &[(&str, &str)] = &[
    ("name", "alice"),
    ("new", "val"),
    ("team", ""),
    ("x", "y"),
    ("quote", "a\"b"),
];

fn dump_info_record(o: &mut String, idx: usize, rec: &[u8]) {
    // The fixture is pure ASCII, so the &str round trip is byte-lossless.
    let s = String::from_utf8_lossy(rec).into_owned();

    let _ = write!(o, "rec {idx} ");
    qstr(o, rec);
    let _ = writeln!(o);
    let _ = writeln!(o, "val {}", Info_Validate(&s) as c_int);

    for vk in VKEYS {
        let v = Info_ValueForKey(&s, vk);
        let _ = write!(o, "vfk ");
        qstr(o, vk.as_bytes());
        let _ = write!(o, " ");
        qstr(o, v.as_bytes());
        let _ = writeln!(o);
    }

    for rk in RKEYS {
        let mut tmp = s.clone();
        Info_RemoveKey(&mut tmp, rk);
        let _ = write!(o, "rk ");
        qstr(o, rk.as_bytes());
        let _ = write!(o, " ");
        qstr(o, tmp.as_bytes());
        let _ = writeln!(o);
    }
    for rk in RKEYS {
        let mut tmp = s.clone();
        Info_RemoveKey_Big(&mut tmp, rk);
        let _ = write!(o, "rkb ");
        qstr(o, rk.as_bytes());
        let _ = write!(o, " ");
        qstr(o, tmp.as_bytes());
        let _ = writeln!(o);
    }
    for (k, v) in SKV {
        let mut tmp = s.clone();
        Info_SetValueForKey(&mut tmp, k, v);
        let _ = write!(o, "svk ");
        qstr(o, k.as_bytes());
        let _ = write!(o, " ");
        qstr(o, v.as_bytes());
        let _ = write!(o, " ");
        qstr(o, tmp.as_bytes());
        let _ = writeln!(o);
    }
    for (k, v) in SKV {
        let mut tmp = s.clone();
        Info_SetValueForKey_Big(&mut tmp, k, v);
        let _ = write!(o, "svkb ");
        qstr(o, k.as_bytes());
        let _ = write!(o, " ");
        qstr(o, v.as_bytes());
        let _ = write!(o, " ");
        qstr(o, tmp.as_bytes());
        let _ = writeln!(o);
    }
    o.push_str("--\n");
}

fn dump_info(o: &mut String) {
    o.push_str("== info ==\n");
    let data = read_fixture("infostrings.txt");
    for (idx, rec) in data.split(|&b| b == 0).enumerate() {
        dump_info_record(o, idx, rec);
    }

    // Big infostring in (MAX_INFO_STRING, BIG_INFO_STRING); exercises the
    // Info_ValueForKey BIG_INFO_STRING guard. Built identically to the C dumper.
    let mut big = String::new();
    let mut i = 0;
    while big.len() < 1100 {
        let _ = write!(big, "\\k{i}\\v{i}");
        i += 1;
    }
    let _ = writeln!(o, "big len {}", big.len());
    let _ = writeln!(o, "big val {}", Info_Validate(&big) as c_int);
    let _ = write!(o, "big vfk k50 ");
    qstr(o, Info_ValueForKey(&big, "k50").as_bytes());
    let _ = writeln!(o);
    let _ = write!(o, "big vfk missing ");
    qstr(o, Info_ValueForKey(&big, "missing").as_bytes());
    let _ = writeln!(o);
}

#[test]
fn qshared_parity() {
    let mut o = String::new();
    dump_clamp(&mut o);
    dump_tokens(&mut o);
    dump_compress(&mut o);
    dump_braced(&mut o);
    dump_skipline(&mut o);
    dump_strhelpers(&mut o);
    dump_va(&mut o);
    dump_sprintf(&mut o);
    dump_info(&mut o);
    o.push_str("== end ==\n");
    compare(env!("CARGO_MANIFEST_DIR"), "qshared", &o);
}
