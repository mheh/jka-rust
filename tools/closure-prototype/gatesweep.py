#!/usr/bin/env python3
"""PROTOTYPE — throwaway. #ifdef-GATE ACTIVATION sweep. Sibling of
constsweep.py / enginesweep.py: verifies every conditional-compilation region in
the jampgame TU set against the OPERATIONAL ground truth (the referee oracle
dylib's preprocessor state) and joins the verdicts against the Rust port.

The bug class it hunts (the `g_log` trap, fixed fe38963d): a porter declared a
whole gate dead ("`LOGGING_WEAPONS` is never #define'd") when the define lives at
`g_log.c:3` — in the .c itself. ~16 functions were stubbed as dead code. This
sweep catches BOTH lying rationale comments AND silent drops with no comment.

Ground truth = the referee oracle build (tools/referee-oracle/build.sh /
~/Developer/jka/seam-test/referee/artifacts/oracle-build.sh): the exact -D set
and compiler the A/B parity dylib is built with. We do not re-derive #if state by
hand — we let the SAME preprocessor resolve it, so same-TU #defines, includes,
and build flags all resolve correctly. Per-region verdicts come from SENTINEL
injection: a unique token is spliced after every conditional directive in a
throwaway copy; whichever sentinels survive preprocessing mark ACTIVE regions
(this is the EFFECTIVE/net verdict — it already accounts for inactive parents).

Phases:
  1  verdicts.json — every conditional region across all game/*.c TUs, its
                     controlling macro/expr, ACTIVE/INACTIVE verdict, and the
                     top-level symbols (fn/var defs) it introduces.
  2a claims.json   — gate-rationale comments in crates/mp/game/src joined to the
                     verdict table -> TRUE or LYING.
  2b drops.json    — ACTIVE-region symbols missing from Rust (candidate SILENT
                     DROP) + INACTIVE-region symbols present in Rust
                     (PORTED-DEAD-CODE) + crude body-size ratios.
  3  worklist.md + audit-draft.md
  +  config-flip: gates whose verdict flips under a presumed retail define set
                  (FINAL_BUILD / MISSIONPACK / _DEBUG / _XBOX / _WIN32 / _JK2),
                  listed separately, never silently resolved.

Usage: python3 gatesweep.py [--out DIR] [--jobs N] [--no-flip]
Outputs land in tools/closure-prototype/out/gatesweep/ (gitignored).
"""
import argparse
import json
import re
import subprocess
import sys
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import closure as C

REPO = C.REPO
ORACLE = C.ORACLE
GAME = ORACLE / "codemp" / "game"
CODEMP = ORACLE / "codemp"
SHIM = REPO / "tools" / "referee-oracle" / "shim" / "oracle_shim.h"
RUST_SRC = REPO / "crates" / "mp" / "game" / "src"

# ---- OPERATIONAL GROUND TRUTH (referee oracle build.sh) ----------------------
# Defines the A/B parity dylib is built with. NOTE this DIFFERS from closure.py's
# jampgame profile (which uses MISSIONPACK + _JK2); the referee uses _JK2MP and
# does NOT define MISSIONPACK. The referee is what parity is measured against, so
# its preprocessor state is authoritative here.
GT_DEFINES = ["QAGAME", "_JK2MP", "__linux__", "_FORTIFY_SOURCE=0", "NDEBUG"]
INCLUDES = ["game", "qcommon", "ghoul2", "cgame", "icarus"]

# Macros whose flip between referee and a presumed retail config matters. If a
# gate's condition references any of these, we probe concrete flips (never
# silently resolve them).
CONFIG_MACROS = {
    "FINAL_BUILD", "_DEBUG", "DEBUG", "NDEBUG", "MISSIONPACK", "_XBOX",
    "_WIN32", "WIN32", "__linux__", "_JK2", "_JK2MP", "DEDICATED", "RELEASE",
    "_CONSOLE", "__APPLE__", "__MACH__", "_GAME", "_M_IX86", "__GNUC__",
}
# Retail-ish toggles: (label, adds, removes). Each re-preprocesses and diffs the
# ACTIVE set against baseline; any region that changes verdict is a config-flip.
FLIP_TOGGLES = [
    ("+FINAL_BUILD", ["FINAL_BUILD"], []),
    ("+MISSIONPACK", ["MISSIONPACK"], []),
    ("+_DEBUG-NDEBUG", ["_DEBUG", "DEBUG"], ["NDEBUG"]),
    ("+_XBOX", ["_XBOX"], []),
    ("+_WIN32-__linux__", ["_WIN32", "WIN32"], ["__linux__"]),
    ("+_JK2-_JK2MP", ["_JK2"], ["_JK2MP"]),
]

SENT = "ZZGATEPROBE"  # sentinel prefix; unique per (tu,clause)

CXX_CANDIDATES = ["g++-16", "g++-15", "g++-14", "g++-13", "g++-12",
                  "/opt/homebrew/bin/g++-16", "g++"]


def find_cxx() -> str:
    for c in CXX_CANDIDATES:
        try:
            out = subprocess.run([c, "--version"], capture_output=True,
                                 text=True, timeout=10)
        except (FileNotFoundError, subprocess.TimeoutExpired):
            continue
        if out.returncode == 0 and "clang" not in out.stdout.lower():
            return c
    sys.exit("gatesweep: no real GCC found (need g++-1x from `brew install gcc`)")


# ============================================================ C text cleaning
def clean_lines(lines):
    """Return per-line text with // and /* */ comments and string/char-literal
    contents blanked, for brace counting and directive/symbol regexes. Preserves
    line count and directive `#` prefixes."""
    out = []
    in_block = False
    for ln in lines:
        res = []
        i, n = 0, len(ln)
        while i < n:
            c = ln[i]
            if in_block:
                if c == "*" and i + 1 < n and ln[i + 1] == "/":
                    in_block = False
                    i += 2
                    continue
                i += 1
                continue
            if c == "/" and i + 1 < n and ln[i + 1] == "/":
                break
            if c == "/" and i + 1 < n and ln[i + 1] == "*":
                in_block = True
                i += 2
                continue
            if c == '"' or c == "'":
                q = c
                res.append(q)
                i += 1
                while i < n:
                    if ln[i] == "\\":
                        i += 2
                        continue
                    if ln[i] == q:
                        break
                    i += 1
                res.append(q)
                i += 1
                continue
            res.append(c)
            i += 1
        out.append("".join(res))
    return out


# ============================================================ directive parse
DIRECTIVE_RE = re.compile(r"^\s*#\s*(ifdef|ifndef|if|elif|else|endif)\b(.*)$")
IDENT_RE = re.compile(r"[A-Za-z_]\w*")


def parse_clauses(clean, tu_id):
    """Walk cleaned lines, emit one region record per #if/#ifdef/#ifndef/#elif/
    #else clause. body_start/body_end are 1-indexed inclusive line ranges of the
    branch body (excluding directive lines)."""
    clauses = []
    stack = []          # list of clause indices (open branch per group)
    group_stack = []    # list of group ids
    next_group = 0
    for idx, raw in enumerate(clean):
        n = idx + 1
        m = DIRECTIVE_RE.match(raw)
        if not m:
            continue
        kind, rest = m.group(1), m.group(2).strip()
        if kind in ("ifdef", "ifndef", "if"):
            gid = next_group
            next_group += 1
            parent = stack[-1] if stack else None
            cl = _mkclause(clauses, tu_id, gid, kind, rest, n, parent,
                           len(group_stack))
            group_stack.append(gid)
            stack.append(cl)
        elif kind in ("elif", "else"):
            if stack:
                clauses[stack[-1]]["body_end"] = n - 1
                gid = group_stack[-1] if group_stack else -1
                parent = None
                # parent is the enclosing clause of this group's opener
                opener = clauses[stack[-1]]
                parent = opener["parent"]
                cl = _mkclause(clauses, tu_id, gid, kind, rest, n, parent,
                               len(group_stack) - 1)
                stack[-1] = cl
            # else: stray directive; ignore
        elif kind == "endif":
            if stack:
                clauses[stack[-1]]["body_end"] = n - 1
                stack.pop()
            if group_stack:
                group_stack.pop()
    # close any dangling
    for ci in stack:
        if clauses[ci]["body_end"] is None:
            clauses[ci]["body_end"] = len(clean)
    return clauses


def _mkclause(clauses, tu_id, gid, kind, cond, dline, parent, depth):
    cid = len(clauses)
    if kind in ("ifdef", "ifndef"):
        toks = IDENT_RE.findall(cond)
        macro = toks[0] if toks else ""
        macros = [macro] if macro else []
    elif kind == "else":
        macros = []
    else:  # if / elif
        macros = [t for t in IDENT_RE.findall(cond) if t != "defined"]
    clauses.append({
        "cid": cid, "tu": tu_id, "group": gid, "kind": kind,
        "cond": cond, "macros": macros,
        "directive_line": dline, "body_start": dline + 1, "body_end": None,
        "parent": parent, "depth": depth,
        "active": None, "symbols": [], "config_sensitive": False,
        "flips": [],
    })
    return cid


# ============================================================ sentinel build
def build_sentinel_copy(lines, clauses, tu_base):
    """Return text of a copy with a unique sentinel token spliced right after
    each clause's directive line. Sentinel = <PREFIX>_<tu>_<cid>_. Injected on
    its own physical line so it never joins a macro invocation's arg list."""
    inject = defaultdict(list)  # after-line -> [tokens]
    tokens = {}
    for cl in clauses:
        tok = f"{SENT}_{tu_base}_{cl['cid']}_"
        tokens[cl["cid"]] = tok
        # a directive may span lines via backslash continuation — inject after
        # the LAST continued line, never inside the condition
        after = cl["directive_line"]
        while after <= len(lines) and lines[after - 1].rstrip().endswith("\\"):
            after += 1
        inject[after].append(tok)
    out = []
    for i, ln in enumerate(lines):
        out.append(ln)
        n = i + 1
        if n in inject:
            for tok in inject[n]:
                out.append(tok)
    return "\n".join(out) + "\n", tokens


def preprocess(cxx, text, extra_adds=(), extra_removes=()):
    """Preprocess `text` (a C TU) with the ground-truth flags (plus toggles).
    Returns (ok, stdout)."""
    defines = [d for d in GT_DEFINES
               if d.split("=")[0] not in set(extra_removes)]
    defines += list(extra_adds)
    cmd = [cxx, "-E", "-x", "c++", "-std=gnu++98", "-fpermissive", "-w",
           "-fsigned-char"]
    for d in defines:
        cmd += ["-D", d]
    cmd += ["-include", str(SHIM)]
    for inc in INCLUDES:
        cmd += ["-I", str(CODEMP / inc)]
    cmd += ["-"]
    try:
        p = subprocess.run(cmd, input=text, capture_output=True, text=True,
                           timeout=120)
    except subprocess.TimeoutExpired:
        return False, ""
    return p.returncode == 0, p.stdout


# ============================================================ symbol extraction
KEYWORDS = {"if", "else", "for", "while", "switch", "return", "do", "sizeof",
            "case", "default", "goto", "break", "continue", "typedef", "struct",
            "union", "enum", "extern", "static", "const", "volatile", "register",
            "unsigned", "signed", "void", "int", "char", "float", "double",
            "long", "short", "inline"}


def extract_toplevel_symbols(clean, active_line):
    """Single pass over ACTIVE, cleaned lines. At brace-depth 0, recognise
    top-level function definitions (`... NAME( ... ) {`) and file-scope variable
    definitions (`... NAME ... ;` / `... NAME[...] = {`). Returns list of
    {name, kind, def_line, end_line}. Heuristic; strings/comments already blanked
    and directive lines skipped. Braces inside inactive branches are ignored."""
    symbols = []
    depth = 0
    decl = ""            # accumulated depth-0 text of the current declaration
    decl_start = None
    pending_fn = None    # (name, start_line) whose body we are scanning
    fn_depth = 0
    for idx, raw in enumerate(clean):
        n = idx + 1
        if not active_line[idx]:
            continue
        if DIRECTIVE_RE.match(raw):
            continue
        j = 0
        L = len(raw)
        while j < L:
            ch = raw[j]
            if ch == "{":
                if depth == 0:
                    # is `decl` a function signature?
                    mm = re.search(r"(\w+)\s*\([^;{]*\)\s*(?:const\s*)?$",
                                   decl.strip())
                    if mm and mm.group(1) not in KEYWORDS:
                        pending_fn = (mm.group(1), decl_start or n)
                    else:
                        # brace-initialised global (array/struct)
                        nm = _last_var_name(decl)
                        if nm and not decl.strip().startswith("extern"):
                            symbols.append({"name": nm, "kind": "var",
                                            "def_line": decl_start or n,
                                            "end_line": n})
                    decl = ""
                    decl_start = None
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth < 0:
                    depth = 0
                if depth == 0 and pending_fn:
                    symbols.append({"name": pending_fn[0], "kind": "fn",
                                    "def_line": pending_fn[1], "end_line": n})
                    pending_fn = None
                    decl = ""
                    decl_start = None
            elif ch == ";":
                if depth == 0:
                    if pending_fn:
                        pending_fn = None
                    else:
                        nm = _last_var_name(decl)
                        # skip bare prototypes (`...)` before `;`) and extern
                        # declarations (references, not definitions to port).
                        if (nm and not decl.rstrip().endswith(")")
                                and not decl.strip().startswith("extern")
                                and not decl.strip().startswith("typedef")):
                            symbols.append({"name": nm, "kind": "var",
                                            "def_line": decl_start or n,
                                            "end_line": n})
                    decl = ""
                    decl_start = None
            else:
                if depth == 0 and not pending_fn:
                    if decl_start is None and ch.strip():
                        decl_start = n
                    decl += ch
            j += 1
        if depth == 0 and not pending_fn and decl.strip():
            decl += " "
    return symbols


def _last_var_name(decl):
    """Name of a file-scope variable from its declaration text (last identifier
    before `=`, `[`, or end). Rejects control keywords."""
    d = decl.strip()
    if not d:
        return None
    # cut at first `=` or `[`
    d = re.split(r"[=\[]", d, maxsplit=1)[0]
    toks = IDENT_RE.findall(d)
    if not toks:
        return None
    nm = toks[-1]
    if nm in KEYWORDS:
        return None
    # need at least a type token before the name
    if len(toks) < 2:
        return None
    return nm


def compute_active_lines(clauses, nlines):
    """Bool per source line (0-indexed): is it in the taken branch of every
    enclosing conditional? A line's immediate clause is the deepest clause whose
    body range contains it; that clause's `active` (effective) verdict decides."""
    active = [True] * nlines
    for cl in clauses:
        if cl["active"]:
            continue
        # inactive clause -> blank its body lines (and directive line)
        for ln in range(cl["directive_line"], (cl["body_end"] or nlines) + 1):
            if 1 <= ln <= nlines:
                active[ln - 1] = False
    return active


def assign_symbols(clauses, symbols, nlines):
    """Attribute each extracted symbol to the innermost clause whose body
    contains its def_line, else 'unconditional'."""
    # innermost clause per line
    owner = [None] * (nlines + 2)
    # sort clauses by depth ascending so deeper overwrite
    for cl in sorted(clauses, key=lambda c: c["depth"]):
        for ln in range(cl["body_start"], (cl["body_end"] or nlines) + 1):
            if 0 <= ln <= nlines + 1:
                owner[ln] = cl["cid"]
    uncond = []
    by_cid = defaultdict(list)
    for s in symbols:
        cid = owner[s["def_line"]] if s["def_line"] <= nlines + 1 else None
        if cid is None:
            uncond.append(s)
        else:
            by_cid[cid].append(s)
    for cl in clauses:
        cl["symbols"] = by_cid.get(cl["cid"], [])
    return uncond


# ============================================================ per-TU driver
def process_tu(cxx, path, do_flip):
    base = path.stem
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    clean = clean_lines(lines)
    clauses = parse_clauses(clean, base)
    sent_text, tokens = build_sentinel_copy(lines, clauses, base)
    ok, out = preprocess(cxx, sent_text)
    if not ok:
        # verdicts from a partial preprocess would mis-read later regions as
        # INACTIVE — refuse rather than emit silently wrong data
        sys.exit(f"gatesweep: preprocess FAILED for {path} — verdicts would be "
                 "unreliable; fix the TU/flags before trusting any output")
    for cl in clauses:
        tok = tokens[cl["cid"]]
        cl["active"] = (tok in out)
    active_line = compute_active_lines(clauses, len(lines))
    symbols = extract_toplevel_symbols(clean, active_line)
    uncond = assign_symbols(clauses, symbols, len(lines))
    # inactive-region symbols (lighter regex over raw body) for ported-dead-code
    for cl in clauses:
        if not cl["active"] and not cl["symbols"]:
            cl["symbols"] = _inactive_region_symbols(
                clean, cl["body_start"], cl["body_end"] or len(lines))
    # config-flip probing
    flip_records = []
    if do_flip:
        sensitive = [cl for cl in clauses
                     if any(m in CONFIG_MACROS for m in cl["macros"])]
        for cl in sensitive:
            cl["config_sensitive"] = True
        touched = set()
        for cl in sensitive:
            touched.update(cl["macros"])
        if sensitive:
            base_active = {cl["cid"]: cl["active"] for cl in clauses}
            for label, adds, removes in FLIP_TOGGLES:
                if not (set(adds) | set(removes)) & (touched | set(adds) | set(removes)):
                    pass
                ok2, out2 = preprocess(cxx, sent_text, adds, removes)
                if not ok2 and not out2:
                    continue
                for cl in sensitive:
                    now = tokens[cl["cid"]] in out2
                    if now != base_active[cl["cid"]]:
                        cl["flips"].append({
                            "toggle": label,
                            "from": "ACTIVE" if base_active[cl["cid"]] else "INACTIVE",
                            "to": "ACTIVE" if now else "INACTIVE"})
    return {
        "tu": base, "path": str(path), "pp_ok": ok,
        "clauses": clauses, "unconditional_symbols": uncond,
        "nlines": len(lines),
    }


def _inactive_region_symbols(clean, start, end):
    """Best-effort symbol names from an INACTIVE branch body (never preprocessed,
    so brace-scan of active code can't see it). Regex for top-of-line fn/var
    definers; low confidence."""
    syms = []
    depth = 0
    for ln in range(start, end + 1):
        if ln - 1 >= len(clean):
            break
        raw = clean[ln - 1]
        if DIRECTIVE_RE.match(raw):
            continue
        if depth == 0:
            m = re.match(r"^\s*(?:static\s+|QDECL\s+|extern\s+)*"
                         r"[A-Za-z_][\w\s\*]*?\b(\w+)\s*\(", raw)
            if m and m.group(1) not in KEYWORDS:
                syms.append({"name": m.group(1), "kind": "fn?",
                             "def_line": ln, "end_line": ln})
            else:
                m2 = re.match(r"^\s*(?:static\s+|const\s+)*"
                              r"[A-Za-z_][\w\s\*]*?\b(\w+)\s*(?:\[[^\]]*\])*\s*=",
                              raw)
                if m2 and m2.group(1) not in KEYWORDS:
                    syms.append({"name": m2.group(1), "kind": "var?",
                                 "def_line": ln, "end_line": ln})
        depth += raw.count("{") - raw.count("}")
        if depth < 0:
            depth = 0
    return syms


# ============================================================ Rust join
def load_rust_index():
    """Read every .rs under crates/mp/game/src once. Return
    (files: {relpath: text}, defined_fns: {name}, symbol_lines: {name:[(file,idx)]},
     all_text_blob)."""
    files = {}
    for p in RUST_SRC.rglob("*.rs"):
        files[str(p.relative_to(RUST_SRC))] = p.read_text(
            encoding="utf-8", errors="replace")
    fn_re = re.compile(r"\bfn\s+(r#)?([A-Za-z_]\w*)")
    static_re = re.compile(r"\b(?:static|const)\s+(?:mut\s+)?([A-Za-z_]\w*)")
    defined = defaultdict(list)  # name -> [(file, kind, lineidx)]
    for rel, txt in files.items():
        for m in fn_re.finditer(txt):
            defined[m.group(2)].append((rel, "fn", txt[:m.start()].count("\n")))
        for m in static_re.finditer(txt):
            defined[m.group(1)].append((rel, "static", txt[:m.start()].count("\n")))
    return files, defined


def rust_symbol_present(name, files, defined):
    """Is `name` defined (fn/static) anywhere in the crate? Fall back to a
    word-boundary text search (covers struct fields / macro-generated)."""
    if name in defined:
        return "defined", defined[name]
    pat = re.compile(r"\b" + re.escape(name) + r"\b")
    hits = [rel for rel, txt in files.items() if pat.search(txt)]
    if hits:
        return "referenced", hits
    return "absent", []


def rust_fn_span(name, files, defined):
    """Rough Rust fn line count for the body-size ratio (heuristic)."""
    for rel, kind, li in defined.get(name, []):
        if kind != "fn":
            continue
        txt = files[rel]
        lines = txt.splitlines()
        # find the fn line, then brace-match
        start = li
        depth = 0
        seen = False
        for k in range(start, len(lines)):
            depth += lines[k].count("{") - lines[k].count("}")
            if "{" in lines[k]:
                seen = True
            if seen and depth <= 0:
                return (rel, k - start + 1)
    return (None, None)


# ============================================================ claims audit
CLAIM_PATTERNS = re.compile(
    r"never\s+#?define|never\s+defined|not\s+compiled|dead\s+code|"
    r"dropped\s+dead|§20|\bgated\b|#if(?:def|ndef)?\b|compiled\s+out|"
    r"stubbed|not\s+built|inactive\s+branch|dead\s+surface|no-op",
    re.IGNORECASE)
# Asserted polarity of the claim about the cited gate.
DEAD_RE = re.compile(
    r"\bdead\b|\bdropped\b|not\s+compiled|never\s+#?define|never\s+defined|"
    r"compiled\s+out|\bstubbed\b|not\s+built|no-?op|\bomitted\b|\bdisabled\b|"
    r"\bexcluded\b|dead\s+surface|inactive|\bskipped\b|undefined\b",
    re.IGNORECASE)
LIVE_RE = re.compile(
    r"compiles?\s+into|is\s+live|are\s+live|live\s+`?#if|ships?\b|shipped\b|"
    r"is\s+compiled|do\s+compile|active\s+branch|stays?\s+live|kept\s+live",
    re.IGNORECASE)
# Non-gate uses to skip (UB shift notes etc.); only skip if NO gate token remains.
CLAIM_SKIP = re.compile(r"undefined behavior|UB[- ]?shift", re.IGNORECASE)
CITE_STOP = {"TODO", "PORT", "NOTE", "UB", "ABI", "FIXME", "PORT-NOTE", "MP",
             "SP", "NULL", "NOT", "AND", "OR", "THE", "API", "DEC", "IEEE",
             "MSVC", "GCC", "LLVM", "FPU", "CVAR", "DLL", "TU", "TUS"}
# Rust files that are slices of one oracle TU (so base-name != TU stem).
TU_ALIAS = {
    "g_init_game": "g_main", "g_shutdown_game": "g_main",
    "game_cvars": "g_main", "game_globals": "g_main",
    "g_local_consts": "g_main", "g_public_consts": "g_main",
    "g_icarus_set_type": "g_ICARUScb", "ai_main_consts": "ai_main",
    "g_nav_consts": "g_nav", "npc_c": "NPC_spawn",
}
# A DEAD claim about macro M is NOT the trap when the block shows the deadness is
# something other than "M is undefined": an `#if 0` sibling, a runtime
# `if(1)return`, an empty/no-op body, a defined-always note, or a counterpart
# (`#else`/`#elif`/`#ifndef`/CGAME) arm.
SUPPRESS_DEAD = re.compile(
    r"#\s*if\s+0|if\s*\(\s*1\s*\)\s*return|\bunreachable\b|"
    r"no-?op|\bempty\b|is\s+always|always\s+(?:defined|true)|not\s+dead|"
    r"live\s+code|are\s+all\s+live|deliberate",
    re.IGNORECASE)
# The porter believed the macro is OFF (the g_log / BOT_STRAFE trap signal).
PORTER_SAYS_OFF = re.compile(
    r"\bundefined\b|not\s+#?defined|never\s+#?defined|isn'?t\s+#?defined|"
    r"n'?t\s+defined|is\s+off\b", re.IGNORECASE)
# Config macros whose drop is a retail-assumption (debug prints etc.), bucketed
# apart from behavioral drops.
RETAIL_MACROS = {"FINAL_BUILD", "_XBOX", "MISSIONPACK", "_DEBUG", "DEBUG",
                 "VM_OR_FINAL_BUILD", "Q3_VM", "_CONSOLE"}


def _comment_blocks(txt):
    """Coalesce maximal runs of comment lines into blocks so a multi-line
    PORT-NOTE is judged with full context (else/ifndef/CGAME counterparts all
    visible together). Returns [(start_line_1indexed, joined_text)]."""
    lines = txt.splitlines()
    blocks = []
    cur, start = [], None
    for i, line in enumerate(lines):
        s = line.strip()
        is_comment = (s.startswith("//") or s.startswith("/*")
                      or s.startswith("*") or s.startswith("*/"))
        if is_comment:
            if start is None:
                start = i + 1
            cur.append(s.lstrip("/*! ").rstrip())
        else:
            if cur:
                blocks.append((start, " ".join(cur)))
            cur, start = [], None
    if cur:
        blocks.append((start, " ".join(cur)))
    return blocks


def audit_claims(files, gates_tu, verdict_by_macro, tu_set):
    """Scan Rust COMMENT BLOCKS for gate-rationale, extract the cited macro and
    the asserted polarity, and join TU-SCOPED (Rust file base -> oracle TU) with
    ifdef/ifndef awareness.

      DEAD claim  -> LYING if the code the porter called dead is actually ACTIVE
                     in the ground-truth build (the g_log / BOT_STRAFE trap).
      LIVE claim  -> LYING if the cited gate has NO active region.
    Counterpart handling: a note that keeps `#ifdef QAGAME` and drops its
    `#else`/`#elif CGAME`/`#ifndef` arm cites QAGAME only descriptively — the
    dead subject is the counterpart (inactive), so it is NOT LYING."""
    claims = []
    for rel, txt in files.items():
        base = Path(rel).name[:-3]  # strip .rs
        tu = base if base in tu_set else TU_ALIAS.get(base)
        if tu not in tu_set:
            tu = None
        for start, block in _comment_blocks(txt):
            if not CLAIM_PATTERNS.search(block):
                continue
            if CLAIM_SKIP.search(block) and not DEAD_RE.search(
                    re.sub(CLAIM_SKIP, "", block)):
                continue
            cited = set(re.findall(r"`([A-Za-z_]\w+)`", block))
            cited |= set(re.findall(
                r"\b([A-Z][A-Z0-9_]{2,})\b", block))
            # keep macro-shaped tokens: has an underscore (e.g. VM_MEMALLOC_DEBUG,
            # FINAL_BUILD) or is a >=4-char all-caps word (QAGAME, CGAME).
            cited = {c for c in cited
                     if c not in CITE_STOP and ("_" in c or len(c) >= 4)}
            dead = bool(DEAD_RE.search(block))
            live = bool(LIVE_RE.search(block))
            # negated-dead / explicit-live wins polarity
            if re.search(r"not\s+dead|live\s+code|are\s+all\s+live", block, re.I):
                polarity = "LIVE"
            else:
                polarity = "LIVE" if (live and not dead) else (
                    "DEAD" if dead else "NONE")
            has_ifndef = bool(re.search(r"#\s*ifndef", block))
            has_ifdef = bool(re.search(r"#\s*ifdef", block))
            has_counterpart = bool(re.search(
                r"#\s*el(?:se|if)|\belse\b|\belif\b|CGAME|cgame", block))
            suppress = bool(SUPPRESS_DEAD.search(block))
            says_off = bool(PORTER_SAYS_OFF.search(block))
            joined = []
            lying = truthy = False
            for mac in cited:
                info = _resolve_gate(mac, tu, gates_tu, verdict_by_macro,
                                     has_ifdef, has_ifndef, has_counterpart,
                                     polarity, suppress, says_off)
                if info is None:
                    continue
                joined.append(info)
                if polarity == "DEAD" and info["subject_active"] > 0:
                    lying = True
                if polarity == "DEAD" and info["subject_active"] == 0:
                    truthy = True
                if polarity == "LIVE" and info["active"] > 0:
                    truthy = True
                if polarity == "LIVE" and info["active"] == 0:
                    lying = True
            if not joined:
                verdict = "UNMATCHED"
            elif polarity == "NONE":
                verdict = "INFO"
            elif lying:
                verdict = "LYING"
            elif truthy:
                verdict = "TRUE"
            else:
                verdict = "INFO"
            # bucket: retail-config assumption vs behavioral drop
            subj_macros = [j["macro"] for j in joined if j["subject_active"] > 0]
            category = "retail-config" if (
                subj_macros and all(m in RETAIL_MACROS for m in subj_macros)
            ) else "behavioral"
            claims.append({"file": rel, "line": start, "text": block[:280],
                           "cited": sorted(cited), "polarity": polarity,
                           "tu_scope": tu, "category": category,
                           "joined": joined, "verdict": verdict})
    return claims


def _resolve_gate(mac, tu, gates_tu, verdict_by_macro, has_ifdef, has_ifndef,
                  has_counterpart, polarity, suppress, says_off):
    """Return {macro, scope, active, inactive, subject_active, subject_kind} or
    None if `mac` is not a real gate macro. `subject_active` is the active-count
    of the branch the DEAD claim actually names. Global-scope (no TU match) never
    yields a LYING (kind attribution is unreliable), so subject_active=0 there."""
    if tu is not None and (tu, mac) in gates_tu:
        g = gates_tu[(tu, mac)]
        scope = f"{tu} (TU-scoped)"
        tu_scoped = True
    elif mac in verdict_by_macro:
        v = verdict_by_macro[mac]
        g = {"ifdef_active": v["active"], "ifdef_inactive": v["inactive"],
             "ifndef_active": 0, "ifndef_inactive": 0,
             "if_active": 0, "if_inactive": 0}
        scope = "global (no TU match)"
        tu_scoped = False
    else:
        return None
    active = (g["ifdef_active"] + g["ifndef_active"] + g["if_active"])
    inactive = (g["ifdef_inactive"] + g["ifndef_inactive"] + g["if_inactive"])
    subject_active = 0
    subject_kind = "any"
    if polarity == "DEAD" and tu_scoped:
        if has_ifndef and not has_ifdef:
            subject_kind = "ifndef"
            subject_active = g["ifndef_active"]
        elif has_counterpart and not says_off:
            # kept the #ifdef arm, dropped the #else/#elif/#ifndef counterpart.
            subject_kind = "counterpart(inactive)"
            subject_active = g["ifndef_active"]
        elif suppress and not says_off:
            # deadness is an #if 0 / if(1)return / empty-body, not this macro.
            subject_kind = "suppressed(non-macro deadness)"
            subject_active = 0
        elif has_ifdef or says_off:
            subject_kind = "ifdef"
            subject_active = g["ifdef_active"] + g["if_active"]
        else:
            subject_kind = "any"
            subject_active = 0 if suppress else active
    return {"macro": mac, "scope": scope, "active": active,
            "inactive": inactive, "subject_active": subject_active,
            "subject_kind": subject_kind}


# ============================================================ main
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(
        REPO / "tools" / "closure-prototype" / "out" / "gatesweep"))
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--no-flip", action="store_true")
    ap.add_argument("--only", default=None, help="comma TU bases for debugging")
    args = ap.parse_args()
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)

    cxx = find_cxx()
    print(f"gatesweep: CXX={cxx}", file=sys.stderr)
    tus = sorted(GAME.glob("*.c"))
    if args.only:
        want = set(args.only.split(","))
        tus = [t for t in tus if t.stem in want]
    print(f"gatesweep: {len(tus)} TUs, defines={GT_DEFINES}", file=sys.stderr)

    results = []
    do_flip = not args.no_flip
    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        futs = {ex.submit(process_tu, cxx, t, do_flip): t for t in tus}
        for fut in futs:
            pass
        for t, fut in [(futs[f], f) for f in futs]:
            pass
        # collect
        for f in list(futs):
            r = f.result()
            results.append(r)
    results.sort(key=lambda r: r["tu"])

    # ---- verdict aggregation ----
    verdict_by_macro = defaultdict(lambda: {"active": 0, "inactive": 0,
                                            "tus": set()})
    # per (tu, macro): active/inactive split by directive kind (ifdef/ifndef/if)
    gates_tu = defaultdict(lambda: {
        "ifdef_active": 0, "ifdef_inactive": 0, "ifndef_active": 0,
        "ifndef_inactive": 0, "if_active": 0, "if_inactive": 0})
    tu_set = {r["tu"] for r in results}
    total_regions = active_regions = inactive_regions = 0
    for r in results:
        for cl in r["clauses"]:
            total_regions += 1
            if cl["active"]:
                active_regions += 1
            else:
                inactive_regions += 1
            kbase = "ifndef" if cl["kind"] == "ifndef" else (
                "ifdef" if cl["kind"] == "ifdef" else "if")
            for mac in cl["macros"]:
                vm = verdict_by_macro[mac]
                if cl["active"]:
                    vm["active"] += 1
                else:
                    vm["inactive"] += 1
                vm["tus"].add(r["tu"])
                key = f"{kbase}_{'active' if cl['active'] else 'inactive'}"
                gates_tu[(r["tu"], mac)][key] += 1

    # ---- Rust join ----
    files, defined = load_rust_index()

    # verdicts.json
    verdicts_json = []
    for r in results:
        for cl in r["clauses"]:
            verdicts_json.append({
                "tu": r["tu"], "cid": cl["cid"], "kind": cl["kind"],
                "cond": cl["cond"], "macros": cl["macros"],
                "directive_line": cl["directive_line"],
                "body": [cl["body_start"], cl["body_end"]],
                "depth": cl["depth"],
                "verdict": "ACTIVE" if cl["active"] else "INACTIVE",
                "config_sensitive": cl["config_sensitive"],
                "flips": cl["flips"],
                "symbols": cl["symbols"],
            })
    (out / "verdicts.json").write_text(json.dumps({
        "ground_truth_defines": GT_DEFINES, "cxx": cxx,
        "tu_count": len(results), "total_regions": total_regions,
        "active_regions": active_regions, "inactive_regions": inactive_regions,
        "regions": verdicts_json,
    }, indent=1))

    # ---- claims audit ----
    vmac_plain = {k: {"active": v["active"], "inactive": v["inactive"],
                      "tus": v["tus"]} for k, v in verdict_by_macro.items()}
    claims = audit_claims(files, dict(gates_tu), vmac_plain, tu_set)
    (out / "claims.json").write_text(json.dumps(claims, indent=1))

    # ---- silent-drop audit ----
    drops = {"silent_drop_candidates": [], "ported_dead_code": []}
    for r in results:
        for cl in r["clauses"]:
            for s in cl["symbols"]:
                if s["kind"].endswith("?"):
                    continue  # low-confidence inactive-scan symbols
                if not cl["active"]:
                    continue
                status, where = rust_symbol_present(s["name"], files, defined)
                if status == "absent":
                    reg_size = (cl["body_end"] or 0) - cl["body_start"] + 1
                    drops["silent_drop_candidates"].append({
                        "tu": r["tu"], "symbol": s["name"], "kind": s["kind"],
                        "oracle_line": s["def_line"],
                        "oracle_symbol_lines": s["end_line"] - s["def_line"] + 1,
                        "gate": cl["cond"] or f"(depth {cl['depth']})",
                        "gate_macros": cl["macros"],
                        "region_lines": reg_size,
                        "severity": "HIGH" if s["kind"] == "fn" else "MED",
                    })
        # unconditional symbols: also worth a missing check (baseline sanity)
    # ported-dead-code: symbols from INACTIVE regions that ARE defined in Rust,
    # REQUIRING the def to live in the same TU's own Rust file(s) — a bare
    # same-name match anywhere else is a cross-file collision (strcat, generic
    # helpers) and is dropped to keep this list trustworthy.
    def _rust_tu(rel):
        b = Path(rel).name[:-3]
        return TU_ALIAS.get(b, b)
    for r in results:
        for cl in r["clauses"]:
            if cl["active"]:
                continue
            for s in cl["symbols"]:
                if s["kind"].endswith("?"):
                    continue  # low-confidence inactive-scan symbol
                nm = s["name"]
                same_tu = [(rel, k) for rel, k, _ in defined.get(nm, [])
                           if k == "fn" and _rust_tu(rel) == r["tu"]]
                if same_tu:
                    rel, cnt = rust_fn_span(nm, files, defined)
                    drops["ported_dead_code"].append({
                        "tu": r["tu"], "symbol": nm,
                        "oracle_line": s["def_line"],
                        "gate": cl["cond"], "gate_macros": cl["macros"],
                        "rust_file": same_tu[0][0], "rust_fn_lines": cnt,
                    })
    (out / "drops.json").write_text(json.dumps(drops, indent=1))

    # ---- config-flip collection ----
    flips = []
    for r in results:
        for cl in r["clauses"]:
            if cl["flips"]:
                flips.append({
                    "tu": r["tu"], "cid": cl["cid"], "cond": cl["cond"],
                    "macros": cl["macros"], "directive_line": cl["directive_line"],
                    "baseline": "ACTIVE" if cl["active"] else "INACTIVE",
                    "flips": cl["flips"],
                })

    # ---- body-size ratio for present ACTIVE fn symbols (heuristic) ----
    truncation = []
    for r in results:
        for cl in r["clauses"]:
            if not cl["active"]:
                continue
            for s in cl["symbols"]:
                if s["kind"] != "fn":
                    continue
                status, where = rust_symbol_present(s["name"], files, defined)
                if status != "defined":
                    continue
                rel, cnt = rust_fn_span(s["name"], files, defined)
                oc = s["end_line"] - s["def_line"] + 1
                if cnt and oc >= 8 and cnt <= max(3, oc * 0.25):
                    truncation.append({
                        "tu": r["tu"], "symbol": s["name"],
                        "oracle_lines": oc, "rust_lines": cnt,
                        "ratio": round(cnt / oc, 2), "rust_file": rel})

    write_worklist(out, results, claims, drops, flips, truncation,
                   total_regions, active_regions, inactive_regions)
    write_audit_draft(out, results, claims, drops, flips, truncation,
                      total_regions, active_regions, inactive_regions,
                      verdict_by_macro, cxx)
    print(f"gatesweep: wrote {out}", file=sys.stderr)
    # concise stdout summary
    lying = [c for c in claims if c["verdict"] == "LYING"]
    print(f"TUs={len(results)} regions={total_regions} "
          f"ACTIVE={active_regions} INACTIVE={inactive_regions} "
          f"claims={len(claims)} LYING={len(lying)} "
          f"silent_drops={len(drops['silent_drop_candidates'])} "
          f"ported_dead={len(drops['ported_dead_code'])} "
          f"flips={len(flips)} trunc={len(truncation)}")


def write_worklist(out, results, claims, drops, flips, truncation,
                   total, active, inactive):
    L = []
    L.append("# gatesweep worklist\n")
    L.append(f"- TUs: {len(results)}")
    L.append(f"- Conditional regions: {total} (ACTIVE {active} / "
             f"INACTIVE {inactive})")
    L.append(f"- Claims scanned: {len(claims)} "
             f"(TRUE {sum(c['verdict']=='TRUE' for c in claims)}, "
             f"LYING {sum(c['verdict']=='LYING' for c in claims)}, "
             f"INFO {sum(c['verdict']=='INFO' for c in claims)}, "
             f"UNMATCHED {sum(c['verdict']=='UNMATCHED' for c in claims)})")
    L.append(f"- Candidate silent drops: {len(drops['silent_drop_candidates'])}")
    L.append(f"- Ported-dead-code: {len(drops['ported_dead_code'])}")
    L.append(f"- Config-flip gates: {len(flips)}")
    L.append(f"- Truncation heuristic hits: {len(truncation)}\n")

    L.append("## LYING claims\n")
    L.append("| file:line | cited | verdict basis | text |")
    L.append("|---|---|---|---|")
    for c in claims:
        if c["verdict"] != "LYING":
            continue
        basis = "; ".join(f"{j['macro']}: {j['active']}A/{j['inactive']}I "
                          f"[{j['scope']}] subj={j['subject_active']}"
                          for j in c["joined"])
        L.append(f"| {c['file']}:{c['line']} | {', '.join(c['cited'])} | "
                 f"{basis} | {c['text'][:120]} |")

    L.append("\n## Candidate silent drops (ACTIVE region symbol missing in Rust)\n")
    L.append("| TU | symbol | kind | gate | oracle:line | sev |")
    L.append("|---|---|---|---|---|---|")
    for d in sorted(drops["silent_drop_candidates"],
                    key=lambda x: (x["severity"] != "HIGH", x["tu"])):
        L.append(f"| {d['tu']} | {d['symbol']} | {d['kind']} | "
                 f"{d['gate'][:40]} | {d['oracle_line']} | {d['severity']} |")

    L.append("\n## Ported-dead-code (INACTIVE region symbol defined in Rust)\n")
    L.append("| TU | symbol | gate | rust file:lines |")
    L.append("|---|---|---|---|")
    for d in drops["ported_dead_code"]:
        L.append(f"| {d['tu']} | {d['symbol']} | {d['gate'][:30]} | "
                 f"{d['rust_file']}:{d['rust_fn_lines']} |")

    L.append("\n## Config-flip gates\n")
    L.append("| TU | line | cond | baseline | flips |")
    L.append("|---|---|---|---|---|")
    for fl in flips:
        fs = "; ".join(f"{x['toggle']}:{x['from']}->{x['to']}"
                       for x in fl["flips"])
        L.append(f"| {fl['tu']} | {fl['directive_line']} | {fl['cond'][:40]} | "
                 f"{fl['baseline']} | {fs} |")

    L.append("\n## Truncation heuristic (ACTIVE fn, Rust much shorter)\n")
    L.append("| TU | symbol | oracle_lines | rust_lines | ratio |")
    L.append("|---|---|---|---|---|")
    for t in sorted(truncation, key=lambda x: x["ratio"]):
        L.append(f"| {t['tu']} | {t['symbol']} | {t['oracle_lines']} | "
                 f"{t['rust_lines']} | {t['ratio']} |")
    (out / "worklist.md").write_text("\n".join(L) + "\n")


def write_audit_draft(out, results, claims, drops, flips, truncation,
                      total, active, inactive, verdict_by_macro, cxx):
    L = []
    L.append("# gatesweep — findings summary (DRAFT for triage)\n")
    L.append("## Method & ground truth\n")
    L.append(f"- Compiler: `{cxx}` (real GCC), `-std=gnu++98`, per the referee "
             "oracle build.")
    L.append(f"- Ground-truth defines: `{' '.join(GT_DEFINES)}` — the exact set "
             "the A/B parity dylib is built with "
             "(`tools/referee-oracle/build.sh`).")
    L.append("- Per-region verdict via sentinel-token injection + real "
             "preprocessing: same-TU `#define`s, includes and flags all resolve "
             "correctly (this is how the `g_log.c:3 LOGGING_WEAPONS` trap is "
             "caught). Verdict is EFFECTIVE (net of inactive parents).")
    L.append("- NOTE: closure.py's jampgame profile uses a DIFFERENT define set "
             "(`MISSIONPACK`, `_JK2`); the referee uses `_JK2MP` and NOT "
             "`MISSIONPACK`. We use the referee set (operational ground truth).\n")

    L.append("## Bucket counts\n")
    L.append(f"- TUs: {len(results)}")
    L.append(f"- Conditional regions: {total} — ACTIVE {active}, "
             f"INACTIVE {inactive}")
    L.append(f"- Rationale claims: {len(claims)} "
             f"(TRUE {sum(c['verdict']=='TRUE' for c in claims)}, "
             f"LYING {sum(c['verdict']=='LYING' for c in claims)}, "
             f"INFO {sum(c['verdict']=='INFO' for c in claims)}, "
             f"UNMATCHED {sum(c['verdict']=='UNMATCHED' for c in claims)})")
    L.append(f"- Candidate silent drops: "
             f"{len(drops['silent_drop_candidates'])} "
             f"(HIGH {sum(d['severity']=='HIGH' for d in drops['silent_drop_candidates'])})")
    L.append(f"- Ported-dead-code: {len(drops['ported_dead_code'])}")
    L.append(f"- Config-flip gates: {len(flips)}")
    L.append(f"- Truncation heuristic hits: {len(truncation)}\n")

    L.append("## Every LYING claim\n")
    L.append("A LYING claim = a rationale comment that declares gated code dead "
             "when the ground-truth build actually compiles it (the g_log "
             "trap). `behavioral` = real game-logic gate; `retail-config` = "
             "code active in the referee build only because a retail flag "
             "(`FINAL_BUILD` etc.) is undefined — usually debug prints, verify "
             "impact but lower priority.\n")
    lying = [c for c in claims if c["verdict"] == "LYING"]
    if not lying:
        L.append("_none_\n")
    for c in sorted(lying, key=lambda x: x.get("category") != "behavioral"):
        basis = "; ".join(f"`{j['macro']}` = {j['active']} ACTIVE / "
                          f"{j['inactive']} INACTIVE [{j['scope']}], "
                          f"subject-branch active={j['subject_active']}"
                          for j in c["joined"])
        L.append(f"- **[{c.get('category','?')}]** `{c['file']}:{c['line']}` "
                 f"({c['polarity']} claim) — {basis}\n  > {c['text']}")

    L.append("\n## Every candidate silent drop (headline)\n")
    if not drops["silent_drop_candidates"]:
        L.append("_none_\n")
    for d in sorted(drops["silent_drop_candidates"],
                    key=lambda x: (x["severity"] != "HIGH", x["tu"])):
        L.append(f"- [{d['severity']}] `{d['tu']}.c` `{d['symbol']}` "
                 f"({d['kind']}, oracle line {d['oracle_line']}, "
                 f"{d['oracle_symbol_lines']} src lines) — gate "
                 f"`{d['gate']}` {d['gate_macros']}")

    L.append("\n## Config-flip gates (NOT resolved — flag only)\n")
    if not flips:
        L.append("_none_\n")
    else:
        by_tog = defaultdict(int)
        for fl in flips:
            for x in fl["flips"]:
                by_tog[x["toggle"]] += 1
        L.append("Per-toggle counts (a gate may flip under several toggles):")
        for tog, n in sorted(by_tog.items(), key=lambda kv: -kv[1]):
            L.append(f"- `{tog}`: {n} gates")
        L.append("\n`+_JK2-_JK2MP` and `+_XBOX` are BUILD-IDENTITY toggles — the "
                 "MP game DLL is firmly `_JK2MP`, non-`_XBOX`, so those flips are "
                 "expected and not action items. The decision-relevant retail "
                 "toggles are **`+FINAL_BUILD`**, **`+MISSIONPACK`**, and "
                 "**`+_DEBUG-NDEBUG`** — gates that would compile differently in a "
                 "retail vs the referee build. Those are enumerated below; the "
                 "full per-gate table is in `worklist.md`.\n")
        RELEVANT = {"+FINAL_BUILD", "+MISSIONPACK", "+_DEBUG-NDEBUG"}
        shown = 0
        for fl in flips:
            rel = [x for x in fl["flips"] if x["toggle"] in RELEVANT]
            if not rel:
                continue
            fs = "; ".join(f"{x['toggle']} -> {x['to']}" for x in rel)
            L.append(f"- `{fl['tu']}.c:{fl['directive_line']}` `{fl['cond']}` "
                     f"(baseline {fl['baseline']}): {fs}")
            shown += 1
        if shown == 0:
            L.append("_(no FINAL_BUILD/MISSIONPACK/_DEBUG-sensitive gates)_")

    L.append("\n## Ported-dead-code (lower severity)\n")
    if not drops["ported_dead_code"]:
        L.append("_none_\n")
    for d in drops["ported_dead_code"]:
        L.append(f"- `{d['tu']}.c` `{d['symbol']}` (gate `{d['gate']}`) is "
                 f"defined in Rust at `{d['rust_file']}` "
                 f"({d['rust_fn_lines']} lines)")

    L.append("\n## Tool limitations (stated honestly)\n")
    L.append("- **Call-site drops are only caught when commented.** The "
             "silent-drop audit checks whether a top-level SYMBOL is missing; "
             "an ACTIVE gate that only wraps a *call* to an already-ported "
             "function (e.g. `BOT_STRAFE_AVOIDANCE`: `BotTrace_Strafe` is "
             "ported but never invoked) leaves the symbol present, so it shows "
             "0 silent drops and is caught only by the claims audit. "
             "Uncommented call-site drops inside active gates are a blind spot.")
    L.append("- **Body-level truncation is NOT covered** beyond the crude "
             "line-count ratio heuristic below: a same-named Rust fn with a "
             "correct signature but a wrong/partial BODY is invisible to this "
             "sweep. Only presence/absence of the top-level symbol and gate "
             "verdicts are authoritative.")
    L.append("- Symbol extraction is a brace-tracking + regex pass (per the "
             "task's allowance), not a full C parser. File-scope var vs. "
             "function-body statements are distinguished by brace depth over "
             "ACTIVE code; INACTIVE-region symbols use a lighter line regex "
             "(marked `fn?`/`var?`, excluded from silent-drop counts).")
    L.append("- Data globals often become struct fields in the Rust port "
             "(threaded state, not same-named statics); a `var` silent-drop "
             "candidate may be a field rename, hence MED severity. Functions "
             "(HIGH) preserve their Raven names verbatim, so an absent fn is a "
             "strong signal.")
    L.append("- Rust presence is an identifier search across the whole "
             "`mp_game` crate (defs first, then word-boundary text); a symbol "
             "present under a macro-generated or re-exported name could read as "
             "absent (false positive) — triage each candidate.")
    L.append("- Config-flip probing toggles one retail-ish macro group at a "
             "time against the referee baseline; it does not model a full "
             "retail preprocessor state, only whether the gate is "
             "config-sensitive and in which direction it moves.")
    L.append("- Truncation heuristic below is advisory only (Rust fn line span "
             "via brace match vs oracle symbol line span).\n")

    L.append("## Truncation heuristic hits (advisory)\n")
    for t in sorted(truncation, key=lambda x: x["ratio"]):
        L.append(f"- `{t['tu']}.c` `{t['symbol']}`: oracle {t['oracle_lines']} "
                 f"lines vs Rust {t['rust_lines']} (ratio {t['ratio']}) "
                 f"@ `{t['rust_file']}`")
    (out / "audit-draft.md").write_text("\n".join(L) + "\n")


if __name__ == "__main__":
    main()
