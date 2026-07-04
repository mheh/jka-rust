#!/usr/bin/env python3
"""PROTOTYPE — pass-3 zero-park port packets. One work order per oracle game
.c file containing its OPEN fns (todo!()-bodied OR carrying a PORT-ESCALATION
marker in the fn's preamble/body). Pass 3 is the final transcription pass: every
fork ruling is settled (12-22), so a porter transcribes + leaves a `// PORT-NOTE`
instead of ever re-parking with `todo!()`.

Each packet carries, per the pass-3 prep agenda:
 (a) pass-3 rulings digest (12-22 + zero-park + PORT-NOTE convention);
 (b) a THREADING DIGEST PER FN — the settled state channel: `ctx: GameContext`
     (game tier) | `PmoveContext` method / `BgState`+`BgTraps`+`GameCallbacks`
     (bg tier) | RNG/qshared — plus the BgTraps methods, GameCallbacks upcalls,
     and cvar/global/master fields it touches;
 (c) the cited verbatim oracle source of each open fn;
 (d) resolved worktree Rust signatures of every out-of-file symbol it calls
     (signatures are LAW — porters do not change them);
 (e) a state-field map, incl. the 38 `Option<EntityId>` stored fields + the
     `ent_id`/`ent_id_opt` seam-helper usage note;
 (f) the va/printf mapping table (inlined from the worktree porter doc);
 (g) EntThink/EntUse/EntSpawn fn-ptr dispatch guidance (enums in ent_fn_enums.rs);
 (h) deferred-by-ruling markers for ICARUS-internal fns (fork 6).

Files with >60 open fns shard by contiguous fn range. Emits out/pass3/packets/
+ out/pass3/manifest.json (+ trial-manifest.json when --trial passed).

Usage:
  .venv/bin/python packets3.py            # full run
  .venv/bin/python packets3.py --only bg_slidemove.c g_missile.c ...   # subset
  .venv/bin/python packets3.py --trial    # also emit trial-manifest.json
"""
import argparse
import json
import re
from pathlib import Path

import pass2lib as L
import closure as C

REPO = L.REPO
GAME = C.ORACLE / "codemp" / "game"
RULINGS = REPO / "docs" / "handoffs" / "jampgame-fork-discovery.md"
VA_DOC = L.WT / "docs" / "porting" / "va-printf-mapping.md"
OUT = L.HERE / "out" / "pass3"
SHARD_MAX = 60

# ---- the 38 stored gentity_t* fields flipped to Option<EntityId> (ruling 22).
# Names verified against the oracle headers (read-only): g_local.h (gentity_t /
# gclient_t), ai_main.h (bot_state_t), b_public.h (gNPC_t). The gNPC list is the
# 12 gentity_t* members present in b_public.h; ruling 22 tallies gNPC as 13
# ("+1") — the sibling field-flip agent owns the authoritative struct edit, this
# list is for porter field-touch detection only.
ENTITYID_FIELDS = {
    "gentity_t": ["parent", "nextTrain", "prevTrain", "target_ent", "chain",
                  "enemy", "lastEnemy", "activator", "teamchain", "teammaster"],
    "gclient_t": ["hook", "team_leader", "leader", "follower"],
    "bot_state_t": ["currentEnemy", "revengeEnemy", "squadLeader", "lastHurt",
                    "lastAttacked", "wantFlag", "touchGoal", "shootGoal",
                    "dangerousObject", "chatObject", "chatAltObject"],
    "gNPC_t": ["touchedByPlayer", "aimingBeam", "eventOwner", "coverTarg",
               "tempGoal", "goalEntity", "lastGoalEntity", "eventualGoal",
               "captureGoal", "defendEnt", "greetEnt", "watchTarget"],
}
FIELD_OWNER = {f: owner for owner, fs in ENTITYID_FIELDS.items() for f in fs}
ALL_ENTID_FIELDS = sorted(FIELD_OWNER)

# ---- GameCallbacks upcall map (ruling 16): Raven game-tier fn -> trait method.
# Source: crates/mp/game/src/bg_channel/game_callbacks.rs (method doc cites).
GC_MAP = {
    "G_Damage": "damage", "G_DamageFromKiller": "damage_from_killer",
    "G_AddEvent": "add_event", "G_Alloc": "alloc", "G_NewString": "new_string",
    "G_PlayEffect": "play_effect", "G_PlayEffectID": "play_effect_id",
    "G_SoundIndex": "sound_index", "G_ModelIndex": "model_index",
    "G_EffectIndex": "effect_index", "G_CheapWeaponFire": "cheap_weapon_fire",
    "Client_CheckImpactBBrush": "client_check_impact_bbrush",
    "G_FlyVehicleSurfaceDestruction": "flyveh_surface_destruction",
    "G_SetAnim": "set_anim", "NPC_SetAnim": "npc_set_anim",
    "G_CanBeEnemy": "can_be_enemy", "BG_GetTime": "get_time",
    "Q3_SetParm": "q3_set_parm",
}
# ---- BgTraps method map (ruling 13): trap_/strap_ symbol -> trait method.
# Source: crates/mp/game/src/bg_channel/bg_traps.rs.
TRAP_MAP = {
    "trap_Trace": "trace", "trap_PointContents": "pointcontents",
    "trap_FS_FOpenFile": "fs_fopen", "trap_FS_Read": "fs_read",
    "trap_FS_Write": "fs_write", "trap_FS_FCloseFile": "fs_fclose",
    "trap_FS_GetFileList": "fs_getfilelist",
    "trap_FX_PlayEffectID": "fx_play_effect_id",
    "trap_SnapVector": "snap_vector", "trap_Cvar_Register": "cvar_register",
    "strap_G2API_GetBoltMatrix": "g2api_get_bolt_matrix",
    "strap_G2API_GetBoltMatrix_NoReconstruct": "g2api_get_bolt_matrix_no_reconstruct",
    "strap_G2API_GetBoltMatrix_NoRecNoRot": "g2api_get_bolt_matrix_no_rec_no_rot",
    "strap_G2API_SetBoneAngles": "g2api_set_bone_angles",
    "strap_G2API_SetBoneAnim": "g2api_set_bone_anim",
    "strap_G2API_GetBoneAnim": "g2api_get_bone_anim",
    "strap_G2API_SetRagDoll": "g2api_set_rag_doll",
    "strap_G2API_AnimateG2Models": "g2api_animate_g2_models",
    "strap_G2API_SetBoneIKState": "g2api_set_bone_ik_state",
    "strap_G2API_IKMove": "g2api_ik_move",
}

VEC = {"vec3_t", "vec5_t", "vec4_t", "vec2_t"}


# --------------------------------------------------------- open-fn scanner
def scan_open():
    """Every worktree game-tier fn record, flagged `open` = todo!()-bodied OR a
    PORT-ESCALATION marker in the fn's preamble comment block or its body."""
    recs = []
    for p in sorted(L.GAME_SRC.rglob("*.rs")):
        fns, text = L.scan_rs_file(p)
        fns.sort(key=lambda r: r["hdr"])
        prev_end = 0
        for r in fns:
            preamble = text[prev_end:r["hdr"]]
            body = text[r["bopen"] + 1:r["bend"]]
            r["esc"] = ("PORT-ESCALATION" in preamble) or ("PORT-ESCALATION" in body)
            r["open"] = r["parked"] or r["esc"]
            r["path"] = p
            recs.append(r)
            prev_end = r["bend"]
    return recs


def numbered_slice(path, a, b):
    lines = path.read_text(errors="replace").splitlines()
    a = max(1, a); b = min(len(lines), b)
    return "\n".join(f"{n:>5} | {lines[n-1]}" for n in range(a, b + 1))


def extract_rulings():
    """Pass-3 rulings digest: the post-mega-pass (8-11) + pass-3 (12-22) sections
    + the bless-the-rule appendix. Skips the blast-radius fork block (1-7) — those
    are settled and their concrete pass-3 form is rulings 12-22."""
    t = RULINGS.read_text()
    start = t.index("## Post-mega-pass rulings")
    return t[start:].rstrip()


# ------------------------------------------------------ per-fn analysis bits
def body_text(cfile, f):
    lines = (GAME / cfile).read_text(errors="replace").splitlines()
    return "\n".join(lines[f["line"] - 1:f["end_line"]])


def strip_comments(src):
    """Drop C // and /* */ comments so token detection ignores commented-out
    code (e.g. a `//if (owner->enemy ...)` line must not flag `enemy`)."""
    src = re.sub(r"/\*.*?\*/", " ", src, flags=re.S)
    src = re.sub(r"//[^\n]*", " ", src)
    return src


TRAP_RE = re.compile(r"\b(s?trap_[A-Za-z0-9_]+)\b")


def traps_in(body):
    return sorted(set(TRAP_RE.findall(body)))


def entid_fields_in(body):
    hit = []
    for fld in ALL_ENTID_FIELDS:
        if re.search(r"(?:->|\.)" + re.escape(fld) + r"\b", body):
            hit.append(fld)
    return hit


def vec3_notes(f):
    outs = []
    for p in f["params"]:
        t = p["type"]; base = t.replace("const", "").strip()
        if base in VEC:
            outs.append((p["name"], t, "const" in t))
    if not outs:
        return None
    lines = []
    for n, t, ro in outs:
        if ro:
            lines.append(f"    - `{n}: {t}` — `const` in the oracle → read-only, keep "
                         f"`[f32;3]` by value.")
        else:
            lines.append(f"    - `{n}: {t}` — C array (decays to a pointer). **The "
                         f"resolved signature above is LAW**: if it kept `{n}` by value "
                         f"(`[f32;3]`), the fn only reads it — port read-only; if it is "
                         f"`&mut [f32;3]` / `Option<&mut [f32;3]>`, it is a fork-9 "
                         f"out-param — write through it (`&mut`+scalar for a "
                         f"VectorNormalize-style mutate+return).")
    return "\n".join(lines)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--only", nargs="*", default=None,
                    help="restrict to these oracle .c files")
    ap.add_argument("--trial", action="store_true",
                    help="also write trial-manifest.json (3 template files)")
    args = ap.parse_args()

    m = L.load_manifest(); F = m["functions"]
    rulings = extract_rulings()
    va_table = VA_DOC.read_text()

    # needs-ctx per (file, name)
    nc = json.load(open(L.HERE / "out" / "pass2" / "needs-ctx.json"))
    ctx_by = {(x["file"], x["name"]): x for x in nc["fns"]}
    ctxfree = set(json.load(open(L.HERE / "out" / "pass2" /
                                 "ctx-free-boundary.json"))["fns"])

    # worktree signatures + open flags
    recs = scan_open()
    wt = {}
    for r in recs:
        wt.setdefault(r["name"], r)          # first record wins (name-unique enough)
    openname = {r["name"] for r in recs if r["open"]}

    # cvar / globals / bg-state field membership (read-only worktree files)
    gc = (L.GAME_SRC / "game_cvars.rs").read_text()
    cvar_fields = set(re.findall(r"pub (\w+): vmCvar_t", gc))
    gg = (L.GAME_SRC / "game_globals.rs").read_text()
    glob_fields = set(re.findall(r"pub (?:r#)?(\w+):", gg))
    bgst = (L.GAME_SRC / "bg_channel" / "bg_state.rs").read_text()
    bgstate_fields = set(re.findall(r"pub (\w+):", bgst))
    MASTER = {"level", "g_entities", "g_clients"}

    byfile = {}
    for f in F:
        byfile.setdefault(f["file"], []).append(f)
    for v in byfile.values():
        v.sort(key=lambda f: f["line"])

    def wt_sig(name):
        r = wt.get(name)
        if r is None:
            return None
        params = re.sub(r"\s+", " ", r["params"]).strip()
        ret = r["ret"].strip()
        return f"pub fn {name}({params}){(' ' + ret) if ret else ''}".rstrip()

    def method_sig(name):
        """LAW form for a bg-tier ctx-threaded fn: the `impl PmoveContext<'_>`
        method shape (`&mut self` prepended, remaining params + ret as resolved).
        The proven precedent is `bg_pmove.rs` (`PM_Friction`, `PM_ClipVelocity`)."""
        r = wt.get(name)
        if r is None:
            return None
        params = re.sub(r"\s+", " ", r["params"]).strip()
        ret = r["ret"].strip()
        selfp = "&mut self" + (", " + params if params else "")
        return f"pub fn {name}({selfp}){(' ' + ret) if ret else ''}".rstrip()

    # per-name tier + needs-ctx (for LAW method-form detection of callees whose
    # own oracle file differs from the packet's): a bg-tier ctx-threaded fn is a
    # PmoveContext method, not a free fn (ruling 12).
    tier_by_name, needs_by_name = {}, {}
    for f in F:
        nm, fl = f["name"], f["file"]
        tier_by_name.setdefault(nm, L.tier(fl))
        needs_by_name.setdefault(nm, ctx_by.get((fl, nm), {}).get("needs_ctx", False))

    def is_pmove_method(name):
        return tier_by_name.get(name) == "bg" and needs_by_name.get(name, False)

    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "packets").mkdir(exist_ok=True)
    manifest_out = []

    targets = sorted(byfile)
    if args.only:
        targets = [c for c in targets if c in set(args.only)]

    for cfile in targets:
        if L.tier(cfile) == "seam":
            continue
        fns = byfile[cfile]
        open_fns = [f for f in fns if f["name"] in openname]
        if not open_fns:
            continue
        tier = L.tier(cfile)
        is_icarus = cfile == "g_ICARUScb.c"
        n_shards = (len(open_fns) + SHARD_MAX - 1) // SHARD_MAX
        for si in range(n_shards):
            chunk = open_fns[si * SHARD_MAX:(si + 1) * SHARD_MAX]
            shard = (si + 1) if n_shards > 1 else None
            base = cfile[:-2]
            o = render_packet(cfile, tier, is_icarus, chunk, shard, n_shards,
                              rulings, va_table, wt, wt_sig, method_sig,
                              is_pmove_method, ctx_by, ctxfree,
                              cvar_fields, glob_fields, bgstate_fields, MASTER)
            fname = base + (f".shard{shard}" if shard else "") + ".md"
            (OUT / "packets" / fname).write_text(o)
            manifest_out.append({
                "file": cfile, "packet": f"packets/{fname}", "tier": tier,
                "fns": len(chunk), "loc": sum(f["loc"] for f in chunk),
                **({"shard": shard, "shards_total": n_shards} if shard else {}),
            })

    manifest_out.sort(key=lambda e: -e["loc"])
    (OUT / "manifest.json").write_text(json.dumps(manifest_out, indent=1))
    n_files = len({e["file"] for e in manifest_out})
    n_shards = sum(1 for e in manifest_out if "shard" in e)
    tot_fns = sum(e["fns"] for e in manifest_out)
    tot_loc = sum(e["loc"] for e in manifest_out)
    print(f"[packets3] {len(manifest_out)} packets / {n_files} files "
          f"({n_shards} sharded), {tot_fns} open fns, {tot_loc:,} LOC "
          f"-> out/pass3/packets/")

    if args.trial:
        want = ["bg_slidemove.c", "g_missile.c", "NPC_AI_Stormtrooper.c"]
        by = {e["file"]: e for e in manifest_out}
        trial = [by[w] for w in want if w in by]
        (OUT / "trial-manifest.json").write_text(json.dumps(trial, indent=1))
        print(f"[packets3] trial-manifest.json: {[e['file'] for e in trial]}")


def render_packet(cfile, tier, is_icarus, chunk, shard, n_shards, rulings,
                  va_table, wt, wt_sig, method_sig, is_pmove_method,
                  ctx_by, ctxfree,
                  cvar_fields, glob_fields, bgstate_fields, MASTER):
    o = []
    title = cfile + (f" — shard {shard}/{n_shards}" if shard else "")
    o.append(f"# PASS-3 PORT PACKET: `{title}`  (tier: **{tier}**)")
    o.append("")
    o.append(f"Fill the **{len(chunk)}** open functions below (each is `todo!()`-bodied "
             "or carries a `PORT-ESCALATION` marker from an earlier pass). Pass 3 is "
             "the FINAL transcription pass: every fork ruling is settled, so **never "
             "re-park** — transcribe the body against the resolved signatures, and where "
             "a genuine question remains leave a one-line `// PORT-NOTE(<subject>): …` "
             "at the site (NOT `todo!()`, NOT `PORT-ESCALATION`).")
    o.append("")
    o.append(f"- open fns in this shard: **{len(chunk)}**  ·  oracle LOC: "
             f"**{sum(f['loc'] for f in chunk)}**  ·  tier: **{tier}**")
    o.append("")

    # ---- channel primer (tier-specific)
    o.append("## STATE CHANNEL (settled — rulings 8/8a/12-16)")
    o.append("")
    if tier == "game":
        o.append("Game-tier fns thread `ctx: GameContext<'_>` as the **first param** "
                 "(fork-8 shape, the `g_init_game` precedent). Reach state through "
                 "`ctx.world` (a `*mut GameWorld`, STATE-D6 leaf reborrows) and the "
                 "engine through `crate::trap::X(ctx.engine, …)` in oracle syscall "
                 "order. `GameContext` is `Copy` — pass `*self`/`ctx` freely.")
    elif tier == "bg":
        o.append("bg-tier fns CANNOT see `GameContext` (bg < game). The settled channel "
                 "(rulings 12-16):")
        o.append("- **pmove working set** (`pm`/`pml`/`pm_entSelf`/`pm_entVeh`/`pm_flying`/"
                 "`gPMDoSlowFall`/`pm_cancelOutZoom`) → methods on **`PmoveContext`** "
                 "(`impl PmoveContext<'_> { fn PM_X(&mut self, …) }`, the proven shape in "
                 "`bg_pmove.rs`); reach the working set via `self.pm`/`self.pml`/… .")
        o.append("- **WRITE-SHAPE (critical, ruling 12):** a bg-tier ctx-threaded fn is an "
                 "`impl PmoveContext<'_>` **method** (`pub fn PM_X(&mut self, …)`), NOT a free "
                 "fn — a bare free fn cannot reach `pm`/`pml`/`BgState`/`BgTraps`/"
                 "`GameCallbacks` without a §B3 static. The `LAW` block under each such fn "
                 "prints the method form (`impl PmoveContext<'_> { … }`); match it exactly. "
                 "When you port the fn, **DELETE the stale free-fn `todo!()` skeleton stub** "
                 "for it (the pre-existing bare-signature stub) — replace it with the method, "
                 "never leave a dead duplicate stub. Peer bg methods are called as "
                 "`self.X(…)` (see `PM_Friction`→`PM_ClipVelocity` in `bg_pmove.rs`).")
        o.append("- **session tables / RNG** → `self.bg` (`&mut BgState`): `bgAllAnims`, "
                 "saber parse buffers, vehicle info arrays, `bg_pool`, `rng`.")
        o.append("- **engine surface** → `self.traps` (`&dyn BgTraps`): trace, "
                 "pointcontents, `fs_*`, the `g2api_*` straps, `fx_*`, `snap_vector`, "
                 "`cvar_register`.")
        o.append("- **upcalls into game** → `self.callbacks` (`&mut dyn GameCallbacks`): "
                 "`damage`, `add_event`, `play_effect`, `alloc`, … (entity refs are "
                 "`c_int` entnums, never `gentity_t*`).")
        o.append("- entity access stays the faithful `baseEnt`/`entSize` overlay "
                 "(`PM_BGEntForNum`, ruling 14); the confined `unsafe` deref lives in the "
                 "pmove methods.")
        o.append("- **gentity-only fields from bg** (`inuse`, `takedamage`, `classname`, "
                 "`r.*`, `client`, …): the overlay pointer is the HEAD of the real "
                 "`gentity_t` — cast it, exactly as Raven does: "
                 "`let g = self.bg_ent_for_num(n) as *mut gentity_t;` then "
                 "`(*g).inuse` etc. inside the existing unsafe. This is the NORMAL "
                 "ruling-14 idiom, not a gap: such fields are NEVER missing_symbols "
                 "and never PORT-NOTEs unless the cast target type itself is in doubt.")
    else:  # qshared
        o.append("qshared-tier helpers are below the bg channel. The only stateful one is "
                 "the fork-3 LCG: it lives in `BgState.rng` (ruling 15) and is reached by "
                 "the bg/game caller, not by an ambient global. Pure math/string helpers "
                 "take/return values (fork-9 vec3 out-param rules apply).")
    o.append("")

    # ---- RNG mapping (ruling 15 — the LCG lives on BgState.rng)
    rng_path = ("`ctx.world.bg_state.rng`" if tier == "game"
                else "`self.bg.rng`" if tier == "bg"
                else "the caller's `bg_state.rng`")
    o.append("## RNG (ruling 15) — Raven `rand`/`srand`/`random`/`crandom`/`*rand` → `BgState.rng`")
    o.append("")
    o.append(f"Every RNG call site routes to the ONE generator on `BgState`: {rng_path}. "
             "NEVER a local recipe, NEVER libc `rand`, NEVER the `rand` crate — every draw is "
             "parity-visible. Raven kept two independent generator states "
             "(`q_math.c:1432` holdrand and `bg_lib.c:763` randSeed); both now live on the one "
             "`Rng` (methods below). API on `Rng` "
             "(`crates/mp/game/src/bg_channel/rng.rs`):")
    o.append("")
    o.append(f"| Raven call | Rust |")
    o.append(f"| --- | --- |")
    for raw, mapped in [
        ("Rand_Init(seed)", "Rand_Init(seed)"), ("flrand(min,max)", "flrand(min, max)"),
        ("Q_flrand(min,max)", "Q_flrand(min, max)"), ("irand(min,max)", "irand(min, max)"),
        ("Q_irand(min,max)", "Q_irand(min, max)"), ("srand(seed)", "srand(seed)"),
        ("rand()", "rand()"), ("random()", "random()"), ("crandom()", "crandom()"),
    ]:
        o.append(f"| `{raw}` | `{rng_path[1:-1]}.{mapped}` |")
    o.append("")
    o.append("(These are all methods on `Rng`; call them through the path above. A bg-tier "
             "`PmoveContext` method reaches it as `self.bg.rng.<m>()`.)")
    o.append("")

    # ---- known seam helpers (in the prelude — do NOT report as missing)
    o.append("## KNOWN SEAM HELPERS (in the prelude — import, never report as missing)")
    o.append("")
    o.append("These are LANDED helpers, re-exported via the crate prelude "
             "(`crates/mp/game/src/cstr_util.rs`). Do NOT report them as missing symbols and "
             "do NOT invent your own `CString`/`&str` conversions — use these exact names:")
    o.append("- `cstr(&str) -> CString` — own a NUL-terminated C string for a syscall; bind it "
             "to a local so it outlives the call, pass `.as_ptr()` where Raven passed a "
             "`char*` (the va/printf table below uses `cstr(&s)` for exactly this).")
    o.append("- `cstr_to_str(*const c_char) -> &str` (`unsafe`) — borrow an engine-supplied "
             "C string as a Rust `&str` (used by the va/printf examples as `cstr_to_str(name)`).")
    o.append("- `write_cstr_field(&mut [c_char], &str)` — write a Rust `&str` into a fixed "
             "`[c_char; N]` struct field with truncation + NUL, replacing a Raven "
             "`Q_strncpyz`/`strcpy` into a char array.")
    o.append("")

    # ---- threading digest per fn
    o.append("## THREADING DIGEST — per open fn")
    o.append("")
    for f in chunk:
        body = strip_comments(body_text(cfile, f))
        info = ctx_by.get((cfile, f["name"]), {})
        needs = info.get("needs_ctx", False)
        traps = traps_in(body)
        bgtrap_methods = sorted({TRAP_MAP[t] for t in traps if t in TRAP_MAP})
        gc_calls = sorted({GC_MAP[c["name"]] for c in
                           f["callees"]["in-module"] + f["callees"]["bg"]
                           if c["name"] in GC_MAP})
        gl = {g["name"] for g in f["globals_read"] + f["globals_write"]}
        cv = sorted(n for n in gl if n in cvar_fields)
        gv = sorted(n for n in gl if n in glob_fields)
        ms = sorted(n for n in gl if n in MASTER)
        bs = sorted(n for n in gl if n in bgstate_fields)
        entflds = entid_fields_in(body)
        fnptr = f.get("fnptr_writes", [])

        o.append(f"### `{f['name']}` — {cfile}:{f['line']}-{f['end_line']} "
                 f"({f['loc']} LOC, wave {f['wave']})")
        # channel line
        if tier == "game":
            if needs:
                ch = "`ctx: GameContext<'_>` first param"
                if f["name"] in ctxfree:
                    ch += " — **BUT** in the ctx-free boundary set (called from bg / " \
                          "stored as a raw fn-ptr): its state comes via the bg channel " \
                          "(GameCallbacks upcall or fn-ptr enum), NOT a ctx param — match " \
                          "its existing worktree signature exactly."
            else:
                ch = "pure fn — **no `ctx`** (touches no traps/globals/needs-ctx callee)"
        elif tier == "bg":
            ch = ("`PmoveContext`/`BgState`/`BgTraps`/`GameCallbacks` (bg channel)"
                  if needs else "pure bg fn — no context needed")
        else:
            ch = ("`BgState.rng` (fork-3 LCG)" if needs else "pure qshared helper")
        o.append(f"- **channel:** {ch}")
        if traps:
            if tier == "bg":
                mapped = ", ".join(f"`{t}`→`self.traps.{TRAP_MAP[t]}`" for t in traps
                                   if t in TRAP_MAP)
                unmapped = [t for t in traps if t not in TRAP_MAP]
                o.append(f"- **BgTraps:** {mapped or '_none mapped_'}"
                         + (f"  ·  _not in BgTraps (escalate if bg-reachable): "
                            f"{', '.join(unmapped)}_" if unmapped else ""))
            else:
                o.append(f"- **traps:** " + ", ".join(f"`{t}`→`trap::{t[len('trap_'):]}"
                         f"(ctx.engine, …)`" for t in traps if t.startswith('trap_'))
                         + ("  ·  straps: " + ", ".join(f"`{t}`" for t in traps
                            if t.startswith('strap_')) if any(t.startswith('strap_')
                            for t in traps) else ""))
        if gc_calls:
            note = " (via `self.callbacks`)" if tier == "bg" else " (ported game bodies)"
            o.append(f"- **GameCallbacks upcalls:** {', '.join('`'+g+'`' for g in gc_calls)}{note}")
        stbits = []
        wprefix = "ctx.world" if tier == "game" else "world (bg: via overlay/callbacks)"
        if ms:
            if tier == "bg":
                stbits.append("entities/level (bg: `g_entities` via `PM_BGEntForNum` "
                              "overlay, `level.time` via `self.callbacks.get_time()`): "
                              + ", ".join('`'+n+'`' for n in ms))
            else:
                stbits.append(f"master `{wprefix}`: " + ", ".join('`'+n+'`' for n in ms))
        if cv: stbits.append(f"cvars `{wprefix}.cvars.*`: " + ", ".join('`'+n+'`' for n in cv))
        if gv: stbits.append(f"globals `{wprefix}.globals.*`: " + ", ".join('`'+n+'`' for n in gv))
        if bs: stbits.append("`BgState` (`self.bg` bg / `ctx.world.bg_state` game): "
                             + ", ".join('`'+n+'`' for n in bs))
        if stbits:
            o.append("- **state fields:** " + "  ·  ".join(stbits))
        if entflds:
            grouped = ", ".join(f"`{fl}`({FIELD_OWNER[fl]})" for fl in entflds)
            o.append(f"- **Option<EntityId> fields (ruling 22):** {grouped} — these are "
                     "`Option<EntityId>`: write `Some(ent_id(base, p))` / `None`, "
                     "identity-compare with `==` (never a pointer/address compare).")
        if fnptr:
            fps = ", ".join(f"`{w['field']}={w['target']}`" for w in fnptr)
            o.append(f"- **fn-ptr dispatch writes:** {fps} — assign the matching "
                     "`EntThink`/`EntUse`/`EntTouch`/`EntDie`/`EntPain`/`EntReached`/"
                     "`EntBlocked` enum variant (see `ent_fn_enums.rs`), not a raw fn ptr.")
        vn = vec3_notes(f)
        if vn:
            o.append("- **fork-9 vec3 params:**")
            o.append(vn)
        if is_icarus and f["name"].startswith("Q3_"):
            o.append("- **fork-6 (ICARUS):** this is a `Q3_*` callback body — port as "
                     "ordinary game logic against the forward-declared "
                     "`interface_export_t` seam (engine-side); do NOT port ICARUS "
                     "internals here.")
        o.append("")

    # ---- oracle source
    o.append("## ORACLE SOURCE (verbatim — transcribe these bodies)")
    o.append("")
    for f in chunk:
        o.append(f"### `{f['name']}` — {cfile}:{f['line']}-{f['end_line']}")
        if is_pmove_method(f["name"]):
            msig = method_sig(f["name"])
            if msig:
                o.append("Resolved worktree signature (LAW — this bg-tier ctx-threaded fn is a "
                         "**`PmoveContext` method**, ruling 12; fill this body, do not change "
                         "the signature). DELETE the stale free-fn `todo!()` skeleton stub for "
                         "this fn — replace it, never leave a dead duplicate:")
                o.append("```rust")
                o.append("impl PmoveContext<'_> {")
                o.append("    " + msig + " { /* PORT-NOTE if needed; port here — "
                         "call peer bg fns as `self.X(…)` */ }")
                o.append("}")
                o.append("```")
        else:
            sig = wt_sig(f["name"])
            if sig:
                o.append("Resolved worktree signature (LAW — fill this body, do not change it):")
                o.append("```rust")
                o.append(sig + " { /* PORT-NOTE if needed; port here */ }")
                o.append("```")
        o.append("```c")
        o.append(numbered_slice(GAME / cfile, f["line"], f["end_line"]))
        o.append("```")
        o.append("")

    # ---- resolved call surface
    callees = set()
    own = {f["name"] for f in chunk}
    for f in chunk:
        for c in f["callees"]["in-module"] + f["callees"]["bg"]:
            callees.add(c["name"])
    callees -= own
    o.append(f"## RESOLVED CALL SURFACE ({len(callees)}) — signatures are LAW, do not explore")
    o.append("")
    o.append("```rust")
    for name in sorted(callees):
        pmethod = is_pmove_method(name)
        sig = method_sig(name) if pmethod else wt_sig(name)
        if sig:
            r = wt.get(name)
            tag = "OPEN" if (r["parked"] or r["esc"]) else "ported"
            note = "  — PmoveContext method: call as `self." + name + "(…)`" if pmethod else ""
            o.append(f"// {tag}: {r['path'].name}{note}")
            o.append(sig + ";")
        else:
            o.append(f"//TODO: Port {name}  // Source: oracle/oracle/codemp/game/ (unresolved)")
    o.append("```")
    o.append("")

    # ---- EntityId helper note
    o.append("## `Option<EntityId>` STORED FIELDS (ruling 22) — seam-helper usage")
    o.append("")
    o.append("The 38 stored `gentity_t*` fields are now `Option<EntityId>` (Raven NULL → "
             "`None`; entity 0 is valid, so no sentinel niche). At the pointer/id seam:")
    o.append("- **store:** `(*ent).enemy = Some(ent_id(base, other))` where `base` is the "
             "`g_entities` array base; `ent_id_opt(base, maybe_null_ptr)` folds a possibly-"
             "NULL pointer straight to `Option<EntityId>`.")
    o.append("- **compare:** `(*ent).enemy == other_id` (id equality) — never an address "
             "compare. `if (*ent).enemy.is_none()` for the Raven `!ent->enemy` test.")
    o.append("- **deref:** resolve the id back to a slot through the arena, not by pointer "
             "arithmetic. `ent_id`/`ent_id_opt`/`EntityId` are in the prelude.")
    o.append("")
    o.append("Full field set by owner:")
    for owner, flds in ENTITYID_FIELDS.items():
        o.append(f"- **{owner}:** {', '.join('`'+x+'`' for x in flds)}")
    o.append("")

    # ---- fn-ptr dispatch guidance
    o.append("## FN-POINTER DISPATCH (fork-2 / ruling 2) — EntThink/EntUse/EntSpawn")
    o.append("")
    o.append("Raven stored bare fn pointers in `ent->think`/`use`/`touch`/`die`/`pain`/"
             "`reached`/`blocked` and compared them by address. Those fields are now "
             "per-field fn-ID enums (`EntThink`, `EntUse`, `EntTouch`, `EntDie`, `EntPain`, "
             "`EntReached`, `EntBlocked`) with `PartialEq` — defined in "
             "`crate::ent_fn_enums`.")
    o.append("- **assign:** `(*ent).think = Some(EntThink::multi_trigger_run)` "
             "(the field is `Option<EntThink>`).")
    o.append("- **compare:** `(*ent).think == Some(EntThink::multi_trigger_run)` replaces "
             "Raven's `ent->think == multi_trigger_run` address compare.")
    o.append("- **call:** go through the central `dispatch_think(ctx, id, self_)` (and "
             "`dispatch_use`/`dispatch_touch`/…) rather than an indirect call.")
    o.append("- **spawn:** classname→spawn dispatch is `EntSpawn` (g_spawn.c spawns[] "
             "table dual, in `ent_fn_enums.rs`).")
    o.append("")

    # ---- va/printf table
    o.append("## va / printf / Com_sprintf MAPPING (ruling 18 — apply verbatim)")
    o.append("")
    o.append(va_table.strip())
    o.append("")

    # ---- rulings digest last (long)
    o.append("## RULINGS DIGEST (verbatim — post-mega-pass 8-11 + pass-3 12-22 + appendix)")
    o.append("")
    o.append(rulings)
    o.append("")
    return "\n".join(o)


if __name__ == "__main__":
    main()
