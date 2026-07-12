#!/usr/bin/env python3
"""PROTOTYPE — throwaway. UNMARKED-PLACEHOLDER-CONST sweep (task #18, phases
1-3). Sibling of fnsweep.py / enginesweep.py: reuses closure.py (module
profiles + unity/header libclang parse) for the ORACLE ground truth, then
mechanically joins it against every integer const declared in the Rust port.

The bug class it hunts: skeleton-phase porting left function-local
`const NAME: c_int = <guessed literal>;` stand-ins (and module-level dupes)
with NO marker comment. They compile and evade marker triage. Confirmed live
examples (fixed 72c25697/8e0cf1aa in g_utils.rs): GT_SIEGE=4 (real 7),
STAT_MAX_HEALTH=1 (real 8), HI_JETPACK=2 (real 7), PMF_FOLLOW=0 (real 4096).

Phases (this script runs 1-3; STOP before phase 4 dispatch):
  1  local_consts.json   — every `const NAME:<int>=<lit>` in crates/mp
                           (fn-local AND module-level) + lying-comment hits.
  2a oracle_values.json  — NAME->value for every enum member + object-like
                           integer #define reachable from the oracle game +
                           shared headers (libclang enums + `clang -dM -E`).
  2b workspace_consts.json — every `pub const NAME:<int>=<lit>` in crates/mp,
                           NAME->[(value, path)] (names collide across modules).
  3  worklist.md         — bucket every phase-1 const:
       SOURCE-CONFLICT (name in BOTH tables, values disagree — canonical is
                        itself wrong; highest priority),
       WRONG-VALUE     (oracle name match, local value differs — live bug),
       SHADOWING       (value correct, canonical exists at a different path),
       HOUSE-NAMED     (name in neither table — judgment),
       CLEAN.

Usage: .venv/bin/python constsweep.py [--out DIR]
Outputs land in tools/closure-prototype/out/constsweep/.
"""
import argparse
import ast
import json
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import closure as C
from clang.cindex import CursorKind

REPO = C.REPO
CRATES_MP = REPO / "crates" / "mp"

# Rust integer types we treat as identity-constant carriers (float/array-size
# consts are out of scope per the plan).
INT_TYPES = {
    "c_int", "c_uint", "c_char", "c_uchar", "c_schar", "c_short", "c_ushort",
    "c_long", "c_ulong", "c_longlong", "c_ulonglong", "c_size_t", "c_ int",
    "i8", "i16", "i32", "i64", "i128", "isize",
    "u8", "u16", "u32", "u64", "u128", "usize",
}
# accept a leading path qualifier on the type (e.g. `libc::c_int`) by matching
# only the final segment. The header regex matches the `const NAME: TYPE =`
# prefix; the RHS (which may span lines) is gathered up to the first `;`.
CONST_HEAD_RE = re.compile(
    r"^(?P<indent>[ \t]*)(?P<pub>pub(?:\([^)]*\))?\s+)?const\s+"
    r"(?P<name>[A-Z][A-Z0-9_]*)\s*:\s*(?P<ty>[A-Za-z_][A-Za-z0-9_:]*)\s*=(?P<rhs>.*)$")
FN_RE = re.compile(r"\bfn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)")
LYING_RE = re.compile(
    r"not yet ported|no canonical|\bplaceholder|\bstand[-\s]?in\b|\bunported",
    re.IGNORECASE)


# ============================================================ literal eval
def strip_int_suffix(tok: str) -> str:
    return re.sub(r"[uUiI](?:8|16|32|64|128|size)$", "",
                  re.sub(r"[uUlL]+$", "", tok))


def eval_rust_literal(rhs: str):
    """Evaluate a simple Rust integer RHS to an int, or None if it references
    identifiers / is too complex. Handles decimal, 0x/0o/0b with `_` digit
    separators, char literals `'0' as c_int`, casts (`... as ty`), unary minus,
    and bitor/shift/arith combinations of literals."""
    s = rhs.strip()
    # drop `as <type>` casts (possibly several)
    s = re.sub(r"\bas\s+[A-Za-z_][A-Za-z0-9_:]*", "", s)
    s = s.strip()
    if not s:
        return None
    # char literal -> ordinal (handles the '0'..'7' digit-char idiom)
    m = re.fullmatch(r"'(\\?.)'", s)
    if m:
        c = m.group(1)
        esc = {"\\0": 0, "\\n": 10, "\\r": 13, "\\t": 9}
        return esc.get(c, ord(c[-1]))
    # tokenize; reject anything that isn't a number / operator / paren / ws
    toks = re.findall(r"0[xX][0-9a-fA-F_]+|0[oO][0-7_]+|0[bB][01_]+|\d[\d_]*"
                      r"|<<|>>|[-+*/%|&^~()]|'\\?.'|\S", s)
    out = []
    for t in toks:
        if re.fullmatch(r"0[xX][0-9a-fA-F_]+|0[oO][0-7_]+|0[bB][01_]+|\d[\d_]*",
                        t):
            out.append(strip_int_suffix(t.replace("_", "")))
        elif t in ("<<", ">>", "+", "-", "*", "/", "%", "|", "&", "^", "~",
                   "(", ")"):
            out.append(t)
        elif re.fullmatch(r"'\\?.'", t):
            c = t[1:-1]
            out.append(str({"\\0": 0, "\\n": 10, "\\r": 13,
                            "\\t": 9}.get(c, ord(c[-1]))))
        else:
            return None  # identifier or unsupported token
    expr = " ".join(out)
    return _safe_int_eval(expr)


_BINOPS = {ast.Add: lambda a, b: a + b, ast.Sub: lambda a, b: a - b,
           ast.Mult: lambda a, b: a * b, ast.Div: lambda a, b: a // b,
           ast.Mod: lambda a, b: a % b, ast.LShift: lambda a, b: a << b,
           ast.RShift: lambda a, b: a >> b, ast.BitOr: lambda a, b: a | b,
           ast.BitAnd: lambda a, b: a & b, ast.BitXor: lambda a, b: a ^ b}


def _safe_int_eval(expr: str):
    if not expr.strip():
        return None
    try:
        node = ast.parse(expr, mode="eval").body
    except SyntaxError:
        return None

    def ev(n):
        if isinstance(n, ast.Constant) and isinstance(n.value, int):
            return n.value
        if isinstance(n, ast.BinOp) and type(n.op) in _BINOPS:
            a, b = ev(n.left), ev(n.right)
            if a is None or b is None:
                return None
            return _BINOPS[type(n.op)](a, b)
        if isinstance(n, ast.UnaryOp):
            v = ev(n.operand)
            if v is None:
                return None
            if isinstance(n.op, ast.USub):
                return -v
            if isinstance(n.op, ast.UAdd):
                return +v
            if isinstance(n.op, ast.Invert):
                return ~v
        return None
    try:
        return ev(node)
    except Exception:
        return None


# ================================================== PHASE 1 — local consts
def enclosing_scope_scan(lines):
    """Yield (line_no, enclosing_fn_or_module, indent) tracking brace depth so
    we can name the fn a const sits in. Heuristic (ignores braces in strings /
    comments) but adequate for rustfmt'd port code -> a human worklist."""
    depth = 0
    fn_stack = []          # (name, body_depth)
    pending_fn = None
    scopes = []            # per-line resolved enclosing name
    for ln in lines:
        # pop fns whose body has closed
        while fn_stack and depth < fn_stack[-1][1]:
            fn_stack.pop()
        scopes.append(fn_stack[-1][0] if fn_stack else "<module>")
        m = FN_RE.search(ln)
        if m:
            pending_fn = m.group("name")
        opens = ln.count("{")
        closes = ln.count("}")
        if opens:
            depth += opens
            if pending_fn is not None:
                fn_stack.append((pending_fn, depth))
                pending_fn = None
        depth -= closes
        if depth < 0:
            depth = 0
        while fn_stack and depth < fn_stack[-1][1]:
            fn_stack.pop()
    return scopes


def iter_consts(lines):
    """Yield (start_idx, pub, name, ty, rhs) for every integer-typed `const`,
    gathering multi-line RHS up to the terminating `;`. ty is the final path
    segment; only INT_TYPES are yielded."""
    i, n = 0, len(lines)
    while i < n:
        m = CONST_HEAD_RE.match(lines[i])
        if not m:
            i += 1
            continue
        ty = m.group("ty").split("::")[-1]
        rhs_parts, j = [m.group("rhs")], i
        # gather until a `;` appears (single-line is the common case)
        while ";" not in rhs_parts[-1] and j + 1 < n and (j - i) < 12:
            j += 1
            rhs_parts.append(lines[j])
        rhs = " ".join(rhs_parts)
        semi = rhs.find(";")
        rhs = (rhs[:semi] if semi >= 0 else rhs).strip()
        if ty in INT_TYPES:
            yield i, bool(m.group("pub")), m.group("name"), ty, rhs
        i += 1


def extract_local_consts():
    consts, lying = [], []
    files = [p for p in CRATES_MP.rglob("*.rs")
             if "/target/" not in str(p)]
    unparsed = []
    for p in sorted(files):
        try:
            text = p.read_text(errors="replace")
        except Exception as e:
            unparsed.append(f"{p}: {e}")
            continue
        lines = text.splitlines()
        scopes = enclosing_scope_scan(lines)
        rel = str(p.relative_to(REPO))
        # comment lines for lying-comment proximity
        comment_lines = {i for i, l in enumerate(lines)
                         if LYING_RE.search(l) and ("//" in l or "/*" in l
                                                    or "*" in l.strip()[:1])}
        for i, is_pub, name, ty, rhs in iter_consts(lines):
            val = eval_rust_literal(rhs)
            entry = {
                "file": rel, "line": i + 1,
                "scope": scopes[i],
                "module_level": scopes[i] == "<module>",
                "pub": is_pub,
                "name": name, "type": ty,
                "rhs": rhs, "value": val,
            }
            consts.append(entry)
            # lying comment within 3 lines of this const?
            near = [j for j in range(max(0, i - 3), min(len(lines), i + 4))
                    if j in comment_lines]
            for j in near:
                lying.append({"file": rel, "const_line": i + 1,
                              "comment_line": j + 1,
                              "name": name,
                              "comment": lines[j].strip()[:160]})
    # also catch lying comments not adjacent to a swept const (bare signatures)
    return consts, lying, unparsed


def extract_all_lying_comments():
    """Every lying-comment hit in crates/mp with its context line, independent
    of const proximity (superset of the near-const ones)."""
    out = []
    for p in sorted(CRATES_MP.rglob("*.rs")):
        if "/target/" in str(p):
            continue
        rel = str(p.relative_to(REPO))
        try:
            lines = p.read_text(errors="replace").splitlines()
        except Exception:
            continue
        for i, line in enumerate(lines):
            if LYING_RE.search(line) and ("//" in line or "/*" in line
                                          or "*" in line.lstrip()[:1]):
                out.append({"file": rel, "line": i + 1,
                            "text": line.strip()[:180]})
    return out


# ============================================== PHASE 2b — workspace consts
def extract_workspace_consts():
    """NAME -> [(value, path)] for every `pub const NAME:<int>=<lit>` in
    crates/mp (multiple modules may declare the same name — keep all)."""
    table = defaultdict(list)
    count = 0
    for p in sorted(CRATES_MP.rglob("*.rs")):
        if "/target/" in str(p):
            continue
        rel = str(p.relative_to(REPO))
        try:
            lines = p.read_text(errors="replace").splitlines()
        except Exception:
            continue
        for i, is_pub, name, ty, rhs in iter_consts(lines):
            if not is_pub:
                continue
            val = eval_rust_literal(rhs)
            table[name].append(
                {"value": val, "path": rel, "line": i + 1, "type": ty})
            count += 1
    return table, count


# ================================================ PHASE 2a — oracle values
# ad-hoc profiles: a single header-TU that pulls in EVERY named game/shared
# header, so all enums + object-like #defines are reachable in one parse.
ORACLE_PROFILES = {
    "constsweep-game": dict(
        lang="c",
        entry=["codemp/game/q_shared.h", "codemp/game/surfaceflags.h",
               "codemp/game/bg_public.h", "codemp/game/bg_weapons.h",
               "codemp/game/anims.h", "codemp/game/teams.h",
               "codemp/game/g_local.h", "codemp/game/bg_local.h",
               "codemp/game/b_local.h", "codemp/game/ai_main.h",
               "codemp/game/w_saber.h"],
        includes=["codemp/game"],
        defines=["NDEBUG", "MISSIONPACK", "QAGAME", "_JK2"]),
}


def _in_codemp(cur) -> bool:
    f = cur.location.file
    if not f:
        return False
    p = f.name.replace("\\", "/")
    return "/codemp/" in p


def collect_enums(tu):
    """NAME -> value for every ENUM_CONSTANT_DECL located in oracle codemp."""
    out = {}
    for cur in tu.cursor.walk_preorder():
        if cur.kind == CursorKind.ENUM_CONSTANT_DECL and _in_codemp(cur):
            try:
                out[cur.spelling] = cur.enum_value
            except Exception:
                pass
    return out


def collect_macro_names(tu):
    """Names of object-like macros DEFINED in oracle codemp headers (so we can
    keep only game/shared macros out of the global -dM dump)."""
    names = set()
    for cur in tu.cursor.walk_preorder():
        if cur.kind == CursorKind.MACRO_DEFINITION and _in_codemp(cur):
            names.add(cur.spelling)
    return names


def _clang_args_for(cfg):
    args = [f"-x{'c++' if cfg['lang'] == 'c++' else 'c'}"]
    if cfg["lang"] == "c++":
        args.append("-std=c++03")
    args += [f"-I{C.SRC_ROOT / inc}" for inc in cfg["includes"]]
    args += [f"-D{d}" for d in cfg["defines"]]
    args += cfg.get("flags", [])
    args += ["-DID_INLINE=inline", "-DMAC_STATIC="]
    args += ["-Wno-everything", "-ferror-limit=0"]
    if sys.platform == "darwin":
        sdk = subprocess.run(["xcrun", "--show-sdk-path"], capture_output=True,
                             text=True).stdout.strip()
        if sdk:
            args.append(f"-isysroot{sdk}")
        res = subprocess.run(["clang", "-print-resource-dir"],
                             capture_output=True, text=True).stdout.strip()
        if res:
            args.append(f"-resource-dir={res}")
    return args


def dump_defines(cfg, scratch: Path):
    """`clang -dM -E` on a synthesized TU including the profile's entry
    headers -> {NAME: body_string} for object-like macros only."""
    entry = cfg["entry"] if isinstance(cfg["entry"], list) else [cfg["entry"]]
    synth = scratch / "constsweep_defs.c"
    synth.write_text("".join(f'#include "{C.SRC_ROOT / e}"\n' for e in entry))
    args = ["clang", "-dM", "-E"] + _clang_args_for(cfg) + [str(synth)]
    res = subprocess.run(args, capture_output=True, text=True)
    defs = {}
    for line in res.stdout.splitlines():
        m = re.match(r"#define\s+(\w+)(\(?)\s*(.*)$", line)
        if not m:
            continue
        name, paren, body = m.group(1), m.group(2), m.group(3)
        if paren == "(":            # function-like macro — skip
            continue
        defs[name] = body.strip()
    return defs, len(res.stderr.splitlines())


# ---- macro body evaluation (recursive substitution + safe int eval)
def eval_macro(name, defs, enums, seen=None):
    if seen is None:
        seen = set()
    if name in enums:
        return enums[name]
    if name not in defs or name in seen:
        return None
    return eval_macro_body(defs[name], defs, enums, seen | {name})


def eval_macro_body(body, defs, enums, seen):
    body = body.split("//")[0]
    body = re.sub(r"/\*.*?\*/", "", body)
    # strip C integer-type casts: (int) (unsigned) (unsigned int) (long) ...
    body = re.sub(r"\(\s*(?:unsigned|signed)?\s*"
                  r"(?:int|char|short|long(?:\s+long)?|unsigned)\s*\)", " ",
                  body)
    body = body.strip()
    if not body:
        return None
    toks = re.findall(r"0[xX][0-9a-fA-F]+[uUlL]*|\d+[uUlL]*|[A-Za-z_]\w*"
                      r"|<<|>>|[-+*/%|&^~()]|'\\?.'|\S", body)
    out = []
    for t in toks:
        if re.fullmatch(r"0[xX][0-9a-fA-F]+[uUlL]*|\d+[uUlL]*", t):
            out.append(strip_int_suffix(t))
        elif re.fullmatch(r"[A-Za-z_]\w*", t):
            v = eval_macro(t, defs, enums, seen)
            if v is None:
                return None
            out.append(str(v))
        elif re.fullmatch(r"'\\?.'", t):
            c = t[1:-1]
            out.append(str({"\\0": 0, "\\n": 10, "\\r": 13,
                            "\\t": 9}.get(c, ord(c[-1]))))
        elif t in ("<<", ">>", "+", "-", "*", "/", "%", "|", "&", "^", "~",
                   "(", ")"):
            out.append(t)
        else:
            return None
    return _safe_int_eval(" ".join(out))


def extract_oracle_values(scratch: Path):
    enums, macro_names, macro_bodies = {}, set(), {}
    stats = {"profiles": [], "enum_members": 0, "macro_defs_total": 0,
             "macro_names_in_codemp": 0}
    profiles = list(ORACLE_PROFILES.items()) + [("mp-engine-ded",
                                                 C.MODULES["mp-engine-ded"])]
    for pname, cfg in profiles:
        C.MODULES.setdefault(pname, cfg)
        tu = C.parse_tu(pname, None, unity=False)
        enums.update(collect_enums(tu))
        macro_names |= collect_macro_names(tu)
        defs, nerr = dump_defines(cfg, scratch)
        macro_bodies.update(defs)
        stats["profiles"].append({"name": pname, "macro_defs": len(defs),
                                  "stderr_lines": nerr})
    # evaluate every codemp-owned object-like macro
    macros = {}
    for name in sorted(macro_names):
        if name in macro_bodies:
            v = eval_macro(name, macro_bodies, enums)
            if v is not None:
                macros[name] = v
    values = {}
    values.update({k: v for k, v in macros.items()})
    # enums win over macros only if a macro shadows an enum name (rare); keep
    # enum authoritative for members.
    for k, v in enums.items():
        values[k] = v
    stats["enum_members"] = len(enums)
    stats["macro_defs_total"] = len(macro_bodies)
    stats["macro_names_in_codemp"] = len(macro_names)
    stats["integer_macros_kept"] = len(macros)
    stats["total_names"] = len(values)
    return values, {"enums": enums, "macros": macros}, stats


# =================================================== PHASE 3 — join/bucket
BUCKETS = ["SOURCE-CONFLICT", "WRONG-VALUE", "SHADOWING", "HOUSE-NAMED",
           "CLEAN"]


def bucket_const(c, oracle, ws):
    name, val, path = c["name"], c["value"], c["file"]
    o_has = name in oracle
    o_val = oracle.get(name)
    ws_entries = ws.get(name, [])
    ws_has = bool(ws_entries)
    ws_vals = [e["value"] for e in ws_entries]
    ws_other_path = any(e["path"] != path for e in ws_entries)

    # 1. canonical (workspace) itself disagrees with the oracle
    if o_has and ws_has and any(v is not None and v != o_val for v in ws_vals):
        return "SOURCE-CONFLICT"
    # 2. this const's own value contradicts the oracle
    if o_has and val is not None and val != o_val:
        return "WRONG-VALUE"
    # value correctness for the remaining buckets
    if o_has:
        val_ok = (val is None) or (val == o_val)
    else:
        val_ok = (val is None) or (val in ws_vals) or True  # no oracle to judge
    # 3. a canonical pub const of this name lives elsewhere and we aren't wrong
    if ws_other_path and val_ok:
        return "SHADOWING"
    # 4. name is unknown to both tables
    if not o_has and not ws_has:
        return "HOUSE-NAMED"
    return "CLEAN"


def join(local, oracle, ws):
    for c in local:
        c["bucket"] = bucket_const(c, oracle, ws)
        c["oracle_value"] = oracle.get(c["name"])
        ws_entries = ws.get(c["name"], [])
        c["canonical"] = [{"value": e["value"], "path": e["path"],
                           "line": e["line"]}
                          for e in ws_entries if e["path"] != c["file"]]
    counts = defaultdict(int)
    for c in local:
        counts[c["bucket"]] += 1
    return counts


# ======================================================= render worklist
def hx(v):
    if v is None:
        return "?"
    if isinstance(v, int) and abs(v) >= 256 and (v & (v - 1)) == 0:
        return f"{v} (0x{v:x})"
    return str(v)


def render_worklist(local, oracle, ws, counts, lying, all_lying,
                    oracle_stats, ws_count, unparsed):
    o = []
    o.append("# Unmarked placeholder-const sweep — worklist (task #18, phases 1-3)\n")
    o.append("Generated by `tools/closure-prototype/constsweep.py`. Mechanical "
             "join of every integer `const` in `crates/mp` against oracle "
             "ground truth (libclang enums + `clang -dM -E` object-like "
             "#defines from the game/shared headers) and the workspace's own "
             "`pub const` canonical table. **Uncommitted.** Phase 4 (judgment "
             "fix wave) is NOT run here — dispatch it from this file.\n")

    o.append("## Extraction stats\n")
    o.append(f"- Local integer consts swept (fn-local + module-level): "
             f"**{len(local)}**")
    o.append(f"- Oracle names (enum members + integer #defines): "
             f"**{oracle_stats['total_names']}** "
             f"({oracle_stats['enum_members']} enum members, "
             f"{oracle_stats['integer_macros_kept']} integer macros kept of "
             f"{oracle_stats['macro_names_in_codemp']} codemp macro names)")
    o.append(f"- Workspace `pub const` canonical entries: **{ws_count}** "
             f"(over {len(ws)} distinct names)")
    o.append(f"- Lying-comment hits near a const: **{len(lying)}**; "
             f"total lying comments in crates/mp: **{len(all_lying)}**\n")

    o.append("## Bucket counts\n")
    o.append("| bucket | count | meaning |")
    o.append("| --- | ---: | --- |")
    meaning = {
        "SOURCE-CONFLICT": "name in BOTH tables, canonical value disagrees with "
                           "oracle — the crate const itself is wrong (check by hand)",
        "WRONG-VALUE": "oracle name match, local value differs — live bug",
        "SHADOWING": "value correct, canonical pub const exists at another path — import instead",
        "HOUSE-NAMED": "name in neither table — judgment call at the oracle usage site",
        "CLEAN": "matches oracle / no better canonical",
    }
    for b in BUCKETS:
        o.append(f"| **{b}** | {counts.get(b, 0)} | {meaning[b]} |")
    o.append("")

    def rows(bucket, note=""):
        items = [c for c in local if c["bucket"] == bucket]
        items.sort(key=lambda c: (c["file"], c["line"]))
        o.append(f"## {bucket} ({len(items)})\n")
        if note:
            o.append(note + "\n")
        if not items:
            o.append("_None._\n")
            return
        o.append("| file:line | scope | name | local value | oracle value | canonical path(s) |")
        o.append("| --- | --- | --- | --- | --- | --- |")
        for c in items:
            can = "; ".join(f"{e['path']}:{e['line']}={hx(e['value'])}"
                            for e in c["canonical"][:3]) or "—"
            o.append(f"| {c['file']}:{c['line']} | `{c['scope']}` | "
                     f"`{c['name']}` | {hx(c['value'])} | "
                     f"{hx(c['oracle_value'])} | {can} |")
        o.append("")

    rows("SOURCE-CONFLICT",
         "**Highest priority.** A workspace `pub const` (canonical) itself "
         "disagrees with the oracle — fixing the local site alone would import "
         "a wrong value. Verify the canonical against the oracle first.")
    rows("WRONG-VALUE",
         "Local literal contradicts the oracle. Each is a probable live bug AND "
         "a marker that the surrounding region was half-ported — re-review the "
         "enclosing fn against the oracle line-by-line (TryUse lesson).")
    rows("SHADOWING",
         "Value is right but a canonical `pub const` already exists elsewhere — "
         "replace the local decl with an import.")
    rows("HOUSE-NAMED",
         "Name is in neither the oracle nor the workspace canonical table "
         "(house-invented, e.g. LAST_USEABLE_WEAPON). Adjudicate at the oracle "
         "usage site.")

    o.append("## Lying comments (placeholder/unported signatures)\n")
    o.append("Comments matching /not yet ported|no canonical|placeholder|"
             "stand-in|unported/i. The classic secondary signature is a comment "
             "falsely claiming no canonical exists while it sits in the "
             "prelude.\n")
    if not all_lying:
        o.append("_None._\n")
    else:
        o.append("| file:line | comment |")
        o.append("| --- | --- |")
        for h in sorted(all_lying, key=lambda x: (x["file"], x["line"]))[:200]:
            o.append(f"| {h['file']}:{h['line']} | {h['text']} |")
        if len(all_lying) > 200:
            o.append(f"\n_… +{len(all_lying) - 200} more (full list in "
                     f"lying_comments.json)._")
        o.append("")

    o.append("## Blind spots (scripts could not parse)\n")
    if unparsed:
        for u in unparsed:
            o.append(f"- {u}")
    else:
        o.append("- No files failed to read.")
    o.append("")
    o.append("_Known extraction limits: RHS expressions referencing other "
             "identifiers (e.g. `(MAX_GENTITIES - 1) as c_int`) evaluate to "
             "`null` and cannot be value-matched; multi-line const declarations "
             "and consts whose `{`/`}`-in-string confuse the brace scanner may "
             "carry an approximate `scope`. Values are host-64-bit (matches the "
             "layout asserts)._\n")
    return "\n".join(o)


# ================================================================== main
def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", default=str(Path(__file__).resolve().parent
                                         / "out" / "constsweep"))
    args = ap.parse_args()
    outdir = Path(args.out)
    outdir.mkdir(parents=True, exist_ok=True)
    scratch = outdir / "_scratch"
    scratch.mkdir(exist_ok=True)

    print("[constsweep] phase 1 — local consts …", file=sys.stderr)
    local, lying, unparsed = extract_local_consts()
    all_lying = extract_all_lying_comments()
    print(f"[constsweep]   {len(local)} local integer consts, "
          f"{len(all_lying)} lying comments", file=sys.stderr)

    print("[constsweep] phase 2a — oracle ground truth …", file=sys.stderr)
    oracle, oracle_detail, ostats = extract_oracle_values(scratch)
    print(f"[constsweep]   {ostats['total_names']} oracle names "
          f"({ostats['enum_members']} enums, {ostats['integer_macros_kept']} "
          f"macros)", file=sys.stderr)
    # sanity check
    expect = {"GT_SIEGE": 7, "STAT_MAX_HEALTH": 8, "SETANIM_TORSO": 1,
              "PMF_FOLLOW": 4096, "HI_JETPACK": 7}
    bad = {k: (oracle.get(k), v) for k, v in expect.items()
           if oracle.get(k) != v}
    cs = oracle.get("CONTENTS_SOLID")
    if cs in (None, 0):
        bad["CONTENTS_SOLID"] = (cs, "nonzero")
    if bad:
        print(f"[constsweep] !! SANITY FAILED: {bad}", file=sys.stderr)
    else:
        print("[constsweep]   sanity OK (GT_SIEGE=7 STAT_MAX_HEALTH=8 "
              "SETANIM_TORSO=1 PMF_FOLLOW=4096 HI_JETPACK=7 CONTENTS_SOLID="
              f"{cs})", file=sys.stderr)

    print("[constsweep] phase 2b — workspace canonical …", file=sys.stderr)
    ws, ws_count = extract_workspace_consts()
    print(f"[constsweep]   {ws_count} pub consts over {len(ws)} names",
          file=sys.stderr)

    print("[constsweep] phase 3 — join …", file=sys.stderr)
    counts = join(local, oracle, ws)

    # ---- write outputs
    (outdir / "local_consts.json").write_text(json.dumps(local, indent=1))
    (outdir / "oracle_values.json").write_text(
        json.dumps({"values": oracle, "detail": oracle_detail,
                    "stats": ostats}, indent=1, sort_keys=True))
    (outdir / "workspace_consts.json").write_text(
        json.dumps({k: v for k, v in sorted(ws.items())}, indent=1))
    (outdir / "lying_comments.json").write_text(
        json.dumps({"near_const": lying, "all": all_lying}, indent=1))
    wl = render_worklist(local, oracle, ws, counts, lying, all_lying,
                         ostats, ws_count, unparsed)
    (outdir / "worklist.md").write_text(wl)

    print(f"[constsweep] buckets: " +
          ", ".join(f"{b}={counts.get(b, 0)}" for b in BUCKETS),
          file=sys.stderr)
    print(f"[constsweep] wrote {outdir}/", file=sys.stderr)


if __name__ == "__main__":
    main()
