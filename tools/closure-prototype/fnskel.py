#!/usr/bin/env python3
"""PROTOTYPE — throwaway. Rust signature-skeleton emitter for the logic port.

Consumes the fnsweep.py manifest and emits FAITHFUL Rust signature stubs — one
`.rs` per oracle `.c` file — each function as a `pub fn` with the house
doc-comment + Source cite and a `todo!("Port <RavenName> — <cite>")` body.
Emits to a STAGING dir (out/skel/); nothing lands in crates/.

"Faithful" = mechanical: pointers -> raw pointers by the simplest sound rule,
`qboolean` stays `qboolean`, void return omitted, Raven names kept verbatim.
The idiom-class decisions (qboolean->bool, out-param T*->&mut T, snake_case
renames, …) are settled in a later user session and become edits to the RULES
TABLE below — then re-run. Type names resolve against the ALREADY-PORTED types
in crates/ (via closure.scan_ported); unresolved types render with a
`//TODO: Port <cbase>` marker carrying the raw C spelling.

Usage:
  .venv/bin/python fnskel.py [--manifest out/jampgame-fn-manifest.json]
                             [--out out/skel] [--refresh-stats out/jampgame-fn-stats.md]
"""
import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import closure as C

# ============================================================================
# ============================ EDITABLE RULES TABLE ==========================
# ============================================================================
# Mechanical C-type -> faithful-Rust-type rules. This block is the single knob
# the later idiom-class user session edits; re-run fnskel.py after editing.
# Each rule is deliberately faithful/conservative today (no bool coercion, no
# out-param inference) so the skeleton mirrors Raven 1:1 until a decision lands.

# --- primitive / builtin scalar spellings -> Rust (via native `types`/`math`).
PRIMITIVE_MAP = {
    "void": "()",                 # only as return; omitted entirely when bare
    "qboolean": "qboolean",       # RULE(idiom): may become `bool` after session
    "int": "c_int",
    "signed int": "c_int",
    "unsigned int": "c_uint",
    "unsigned": "c_uint",
    "char": "c_char",
    "signed char": "c_schar",
    "unsigned char": "c_uchar",
    "short": "c_short",
    "short int": "c_short",
    "unsigned short": "c_ushort",
    "long": "c_long",
    "unsigned long": "c_ulong",
    "long long": "c_longlong",
    "unsigned long long": "c_ulonglong",
    "float": "f32",
    "double": "f64",
    "long double": "f64",
    "_Bool": "bool",
    "bool": "bool",
    "size_t": "usize",
    # Raven scalar typedefs that live in native crates (kept faithful by name)
    "byte": "byte",
    "vec_t": "vec_t",
    "qhandle_t": "qhandle_t",
    "fileHandle_t": "fileHandle_t",
    "sfxHandle_t": "sfxHandle_t",
    "clipHandle_t": "clipHandle_t",
    "vec2_t": "vec2_t",
    "vec3_t": "vec3_t",
    "vec4_t": "vec4_t",
    "vec5_t": "vec5_t",
    "matrix3_t": "matrix3_t",
    "va_list": "va_list",
}

# --- pointer rendering RULE. Simplest sound faithful rule: raw pointers, with
#     const-ness taken from the C `const` qualifier on the pointee.
#     RULE(idiom): out-params (`T *` written-through) may later become `&mut T`,
#     in-string `const char *` may become `&CStr`, etc. — edit here.
def render_pointer(base_rust, depth, pointee_const):
    inner = base_rust
    for i in range(depth):
        qual = "const" if (i == 0 and pointee_const) else "mut"
        inner = f"*{qual} {inner}"
    return inner

# --- void handling RULE: a bare `void` return emits NO `-> ...`; `void *`
#     stays a real pointer to `c_void`.
VOID_BASE_RUST = "c_void"

# --- name RULE: keep Raven identifiers verbatim (mechanical); the file carries
#     `#![allow(non_snake_case)]`. RULE(idiom): a session may switch to
#     snake_case renames — implement in `rust_fn_name` below.
def rust_fn_name(raven_name):
    return raven_name

# ============================================================================
# ========================== end of RULES TABLE ==============================
# ============================================================================

STRUCT_KW = re.compile(r"^\s*(?:const\s+)?(?:struct|union|enum)\s+")

# Rust keywords a C param name may collide with — raw-ify mechanically.
RUST_KEYWORDS = {
    "as", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
    "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self",
    "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
    "where", "while", "async", "await", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
}


def safe_param_name(pname):
    if not pname:
        return "_arg"
    if pname in ("self", "Self"):
        return "self_"
    if pname in RUST_KEYWORDS:
        return f"r#{pname}"
    return pname


def resolve_base(cbase, ported):
    """(rust_name, unported_marker_or_None) for a bare C base type name."""
    if cbase in PRIMITIVE_MAP:
        return PRIMITIVE_MAP[cbase], None
    # strip a struct/union/enum tag keyword the spelling may carry
    bare = re.sub(r"^(?:struct|union|enum)\s+", "", cbase).strip()
    if bare in PRIMITIVE_MAP:
        return PRIMITIVE_MAP[bare], None
    st = ported.get(bare)
    if st and st[0] == "ported":
        return bare, None            # Rust decl carries the Raven typedef name
    # unresolved / not-yet-ported: keep raw spelling, flag it
    return bare, bare


def map_ctype(cspelling, ported, *, as_return=False):
    """Faithful Rust rendering of a C type spelling.
    Returns (rust_text, unported_cbase_or_None). For a bare `void` return this
    returns (None, None) meaning 'omit the arrow'."""
    s = cspelling.strip()
    # array param decay is already pointer in clang spellings; handle any [] too
    s = re.sub(r"\[\s*\d*\s*\]", " *", s)
    depth = s.count("*")
    s_noptr = s.replace("*", "").strip()
    # const applies to the pointee if it appears (faithful, simplest reading)
    pointee_const = bool(re.search(r"\bconst\b", s_noptr))
    cbase = re.sub(r"\bconst\b", "", s_noptr).replace("volatile", "").strip()
    cbase = re.sub(r"\s+", " ", cbase)

    if cbase == "void":
        if depth == 0:
            return (None, None) if as_return else ("()", None)
        rust = render_pointer(VOID_BASE_RUST, depth, pointee_const)
        return rust, None

    rust_base, unported = resolve_base(cbase, ported)
    if depth == 0:
        return rust_base, unported
    return render_pointer(rust_base, depth, pointee_const), unported


def render_fn(fn, ported):
    name = fn["name"]
    rust_name = rust_fn_name(name)
    cite = f"oracle/oracle/codemp/game/{fn['file']}:{fn['line']}-{fn['end_line']}"
    src_line = f"oracle/oracle/codemp/game/{fn['file']}:{fn['line']}"
    unported = set()

    o = []
    o.append(f"/// Raven `{name}`.")
    o.append("///")
    o.append(f"/// Source: `{cite}`")
    # params
    param_lines = []
    for p in fn["params"]:
        pname = safe_param_name(p["name"])
        rust_t, un = map_ctype(p["type"], ported)
        if un:
            unported.add(un)
            param_lines.append(f"    //TODO: Port {un}  (C: `{p['type']}`)")
        param_lines.append(f"    {pname}: {rust_t},")
    if fn.get("variadic"):
        param_lines.append("    // variadic `...` — C varargs, seam decision pending")

    ret_rust, un = map_ctype(fn["ret_type"], ported, as_return=True)
    if un:
        unported.add(un)
    ret = f" -> {ret_rust}" if ret_rust is not None else ""

    attrs = ""
    if not param_lines:
        o.append(f"pub fn {rust_name}(){ret} {{")
    else:
        o.append(f"pub fn {rust_name}(")
        o.extend(param_lines)
        o.append(f"){ret} {{")
    o.append(f'    todo!("Port {name} — {src_line}")')
    o.append("}")
    return "\n".join(o), unported


def render_file(cfile, fns, ported):
    o = []
    o.append(f"//! FAITHFUL signature skeleton for `oracle/oracle/codemp/game/{cfile}`.")
    o.append("//!")
    o.append("//! Generated by `tools/closure-prototype/fnskel.py` from the fnsweep")
    o.append("//! manifest. STAGING ONLY — not wired into crates/. Bodies are")
    o.append('//! `todo!()`; types resolve against already-ported crates, unresolved')
    o.append("//! ones carry `//TODO: Port <type>` markers. Re-run after editing the")
    o.append("//! RULES TABLE in fnskel.py.")
    o.append("#![allow(non_snake_case, unused, clippy::all)]")
    o.append("")
    all_unported = set()
    for fn in fns:
        txt, un = render_fn(fn, ported)
        all_unported |= un
        o.append(txt)
        o.append("")
    header_note = ""
    if all_unported:
        header_note = ("// Unported types referenced in this file (need porting "
                       "before this compiles):\n// "
                       + ", ".join(sorted(all_unported)) + "\n\n")
    return o[0:8], header_note, "\n".join(o[8:]).rstrip() + "\n", all_unported


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    here = Path(__file__).resolve().parent
    ap.add_argument("--manifest", default=str(here / "out" / "jampgame-fn-manifest.json"))
    ap.add_argument("--out", default=str(here / "out" / "skel"))
    ap.add_argument("--refresh-stats", default=str(here / "out" / "jampgame-fn-stats.md"))
    ap.add_argument("--mode", default="mp")
    ap.add_argument("--crate-seg", default="game")
    args = ap.parse_args()

    manifest = json.loads(Path(args.manifest).read_text())
    ported, _ = C.scan_ported(args.mode, args.crate_seg)

    by_file = defaultdict(list)
    for fn in manifest["functions"]:
        by_file[fn["file"]].append(fn)

    outdir = Path(args.out)
    outdir.mkdir(parents=True, exist_ok=True)
    file_unported = {}
    total_params = total_unresolved_params = 0
    for cfile, fns in sorted(by_file.items()):
        fns.sort(key=lambda f: f["line"])
        head, note, body, unp = render_file(cfile, fns, ported)
        text = "\n".join(head) + "\n\n" + note + body
        rs = outdir / (cfile[:-2] + ".rs")
        rs.write_text(text)
        file_unported[cfile] = unp
        for fn in fns:
            for p in fn["params"]:
                total_params += 1
                _, un = map_ctype(p["type"], ported)
                if un:
                    total_unresolved_params += 1

    print(f"[fnskel] wrote {len(by_file)} skeleton files to {outdir}")
    print(f"[fnskel] {total_unresolved_params}/{total_params} params reference "
          f"unported types")

    # ---- 3 sample renderings (small / medium / large .c) into the stats md
    sizes = sorted(by_file.items(), key=lambda kv: sum(f["loc"] for f in kv[1]))
    picks = []
    if sizes:
        picks = [("small", sizes[len(sizes) // 12][0]),
                 ("medium", sizes[len(sizes) // 2][0]),
                 ("large", sizes[-1][0])]
    samples = []
    for label, cfile in picks:
        txt = (outdir / (cfile[:-2] + ".rs")).read_text()
        # trim large files to first ~60 lines for the eyeball sample
        lines = txt.splitlines()
        shown = "\n".join(lines[:60])
        if len(lines) > 60:
            shown += f"\n// … +{len(lines) - 60} more lines"
        samples.append((f"{label} — {cfile}", str((outdir / (cfile[:-2] + '.rs'))), shown))

    if args.refresh_stats and Path(args.refresh_stats).exists():
        md = Path(args.refresh_stats).read_text()
        marker = "\n## Sample skeleton renderings"
        if marker in md:
            md = md[:md.index(marker)].rstrip() + "\n"
        add = ["", "## Sample skeleton renderings (fnskel.py)", "",
               "Three files eyeballed for shape — small, medium, large. Full set "
               "in `out/skel/`.", ""]
        for label, path, text in samples:
            add.append(f"### {label}")
            add.append(f"`{path}`")
            add.append("")
            add.append("```rust")
            add.append(text)
            add.append("```")
            add.append("")
        Path(args.refresh_stats).write_text(md.rstrip() + "\n" + "\n".join(add) + "\n")
        print(f"[fnskel] refreshed samples in {args.refresh_stats}")


if __name__ == "__main__":
    main()
