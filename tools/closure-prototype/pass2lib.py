"""PROTOTYPE — pass-2 shared helpers: needs-ctx closure + worktree Rust fn scan."""
import json, re
from pathlib import Path
from collections import defaultdict

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
WT = REPO / ".claude" / "worktrees" / "agent-a43cc53200d2fdf54"
GAME_SRC = WT / "crates" / "mp" / "game" / "src"
MANIFEST = HERE / "out" / "jampgame-fn-manifest.json"

def load_manifest():
    return json.load(open(MANIFEST))

def compute_needs_ctx(F):
    def seed(f):
        return bool(f['callees']['syscall'] or f['globals_read'] or f['globals_write'])
    need = {f['usr'] for f in F if seed(f)}
    callees = {f['usr']: set(f['callee_usrs']) for f in F}
    changed = True
    while changed:
        changed = False
        for f in F:
            if f['usr'] in need:
                continue
            if callees[f['usr']] & need:
                need.add(f['usr']); changed = True
    return need, {f['usr'] for f in F if seed(f)}

# ---- worktree Rust fn scanner
# `^[ \t]*` so INDENTED `impl`-block methods are captured too, not just
# column-0 free fns (the reality guard needs to see ported PmoveContext/BgState
# methods, whose real shape must outrank the classifier's prediction).
FN_RE = re.compile(r'(?m)^[ \t]*(?:pub(?:\([\w:]+\))?\s+)?(?:extern\s+"C"\s+)?fn\s+([A-Za-z_]\w*)\s*\(')
IMPL_RE = re.compile(r'(?m)^[ \t]*impl(?:\s*<[^>]*>)?\s+(?:[\w:<>\', ]+\s+for\s+)?([A-Za-z_]\w*)')
SELF_RE = re.compile(r'^\s*&?\s*(?:mut\s+)?self\b')

def _impl_spans(text):
    """List of (open_brace_idx, close_brace_idx, receiver_type) for each `impl`
    block, so a fn record can report the type it is a method of."""
    spans = []
    for m in IMPL_RE.finditer(text):
        brace = text.find('{', m.end())
        if brace == -1:
            continue
        d = 0; e = brace
        while e < len(text):
            c = text[e]
            if c == '{': d += 1
            elif c == '}':
                d -= 1
                if d == 0:
                    break
            e += 1
        spans.append((brace, e, m.group(1)))
    return spans

def scan_rs_file(path):
    """Return list of fn records: name, params, ret, header_start(char idx of
    'pub fn'), body_open(idx of '{'), body_end(matching '}'), is_todo, has_ctx,
    is_method (leading `self` param), impl_ty (enclosing `impl` receiver or None),
    text slice of signature."""
    text = path.read_text()
    impls = _impl_spans(text)
    out = []
    for m in FN_RE.finditer(text):
        name = m.group(1)
        # find matching ')' for the arg list
        i = m.end() - 1  # at '('
        depth = 0; j = i
        while j < len(text):
            c = text[j]
            if c == '(': depth += 1
            elif c == ')':
                depth -= 1
                if depth == 0:
                    break
            j += 1
        params_text = text[i+1:j]
        # after ')': optional -> ret, then '{'
        k = j + 1
        brace = text.find('{', k)
        ret = text[k:brace].strip()
        # matching close brace of body
        d = 0; e = brace
        while e < len(text):
            c = text[e]
            if c == '{': d += 1
            elif c == '}':
                d -= 1
                if d == 0:
                    break
            e += 1
        body = text[brace+1:e]
        # parked = body's only non-comment content is a single todo!() call.
        nocomment = re.sub(r'//[^\n]*', '', body)
        nocomment = re.sub(r'/\*.*?\*/', '', nocomment, flags=re.S).strip()
        parked = (nocomment.startswith('todo!(') and nocomment.endswith(')')
                  and nocomment.count('todo!(') == 1)
        has_ctx = bool(re.match(r'\s*ctx\s*:\s*GameContext', params_text))
        is_method = bool(SELF_RE.match(params_text))
        impl_ty = None
        for bo, be, ty in impls:
            if bo < m.start() < be:
                impl_ty = ty  # innermost wins (spans are source-ordered)
        out.append(dict(name=name, params=params_text, ret=ret,
                        hdr=m.start(), popen=i, pclose=j, bopen=brace, bend=e,
                        parked=parked, has_ctx=has_ctx,
                        is_method=is_method, impl_ty=impl_ty))
    return out, text

def scan_worktree():
    """name -> list of (rs_path, record). (names can repeat across files.)"""
    recs = []
    for p in sorted(GAME_SRC.rglob("*.rs")):
        fns, _ = scan_rs_file(p)
        for r in fns:
            r['path'] = p
            recs.append(r)
    return recs

QSHARED_FILES = {'q_math.c', 'q_shared.c'}

def tier(cfile):
    """Crate tier of a Raven game .c file (fork 8a/3/11 boundary)."""
    if cfile in QSHARED_FILES or cfile == 'bg_lib.c':
        return 'qshared'
    if cfile.startswith('bg_'):
        return 'bg'
    if cfile == 'g_syscalls.c':
        return 'seam'
    return 'game'

def needs_ctx_report(F):
    """Full per-fn ctx classification. Returns dict."""
    need, seed = compute_needs_ctx(F)
    fns = []
    for f in F:
        t = tier(f['file'])
        n = f['usr'] in need
        why = []
        if f['callees']['syscall']:
            why.append('trap')
        if f['globals_read']:
            why.append('global-read')
        if f['globals_write']:
            why.append('global-write')
        seeded = f['usr'] in seed
        fns.append({
            'name': f['name'], 'file': f['file'], 'tier': t,
            'needs_ctx': n, 'seeded': seeded,
            'why': why if seeded else (['transitive'] if n else []),
            # ctx channel per fork rulings 8/8a/3
            'ctx_kind': ('game' if (n and t == 'game') else
                         'pmove-or-bg' if (n and t == 'bg') else
                         'rng-or-qshared' if (n and t == 'qshared') else
                         'none'),
        })
    return {'need_usrs': sorted(need), 'seed_usrs': sorted(seed), 'fns': fns}
