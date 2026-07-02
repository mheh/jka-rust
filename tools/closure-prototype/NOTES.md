# closure-prototype — PROTOTYPE, throwaway

**Question:** can libclang, given per-module compile flags, mechanically produce
(a) the transitive dependency closure of a Raven type or function, (b) by-value
vs pointer-only classification, (c) ported/unported status vs `crates/`, and
(d) ground-truth size/offset layout data — killing the recurring
"port a type, discover deferred sub-structs mid-port" problem?

**Verdict: YES on all four.** Validated against every already-cargo-verified
assert number, on both trees:

| Case | Tool says | Verified assert | Match |
|---|---:|---:|---|
| MP `saberInfo_t` | 2156 | 2156 | ✓ (+ all field offsets) |
| SP `saberInfo_t` (C++ parse) | 1952 | 1952 | ✓ |
| MP `gclient_s` | 7344 | 7344 | ✓ |
| MP `level_locals_t` | 47176 | 47176 | ✓ |
| MP/SP `saberTrail_t`, `bladeInfo_t` | 116/92, 204/164 | same | ✓ |

Demo runs that show the anti-deferral payoff:
- `mp-game Vehicle_t` → exactly 2 small unported by-value structs
  (`vehWeaponStatus_t` 16B, `vehTurretStatus_t` 20B) + 2 legitimately
  pointer-only deps (`bgEntity_t`, `vehicleInfo_t`). Complete work list up front.
- `mp-game gNPC_t` → 4 unported enums + `gNPCstats_t` (68B); all else ported.
- `mp-game fn:G_Spawn --file codemp/game/g_utils.c` → callees
  (`G_InitGentity`, `G_Error`, `trap_LocateGameData`, `G_SpewEntList`) +
  referenced types with status. Function closures work — this is what the
  logic-porting phase needs.

**Run:** `.venv/bin/python closure.py <module> <symbol> [--layout|--asserts]`
(`--list-modules` for the flag table; `fn:Name --file <oracle-rel.c>` for functions).

## What was learned (keep these decisions)

1. **Per-module flags come from the vcproj Release configs**: MP game
   `MISSIONPACK;QAGAME;_JK2`, cgame `CGAME`, ui `UI_EXPORTS`, SP `_IMMERSION`;
   `NDEBUG` everywhere. WIN32/_WINDOWS deliberately omitted — they gate
   macros/inline asm only, not layouts, so host-64-bit parse matches the
   existing `#[cfg(target_pointer_width = "64")]` asserts. Supply
   `-DID_INLINE=inline -DMAC_STATIC=` since no platform section is active.
2. **MP parses as C, SP as C++ (`-std=c++03`)** — both work with the pip
   `libclang` bundle + `xcrun --show-sdk-path` sysroot + host clang
   `-print-resource-dir`.
3. **Tag→typedef aliasing matters**: clang sees `struct playerState_s`, the
   Rust port declares `playerState_t`; ported-status must check both.
4. **Residual parse errors are benign** (Raven redeclares `powf`, etc.) and
   don't affect record layout; keep printing the warning count.
5. Ported-status is a grep heuristic (`pub struct/enum/union/type X`) with
   own-mode > native > other-mode ranking; good enough.

## v2 additions (tree mode + OpenJK profile)

- `--tree` prints a field-labeled dependency hierarchy (by-value children
  recursed, `*` pointer fields shown as opaque-ok leaves, repeats marked
  "↑ expanded above"), ending with a one-line unported summary.
- `--source openjk --root <checkout>` parses JACoders/OpenJK with its
  CMakeLists flags (`_GAME`/`_CGAME`/`UI_BUILD`/`SP_GAME`, includes rooted at
  `codemp/`+`shared/`, `-std=c++11` for SP). **Verified working** on a shallow
  clone in the session scratchpad.
- **Finding from the OpenJK run:** OpenJK diverged from Raven 1.01 in
  game-private structs — `gclient_s` 7432B (oracle 7344), `clientPersistant_t`
  360B (156), `clientSession_t` 164B (284); saber types moved from
  `q_shared.h` to `bg_public.h`. The ABI-crossing structs still match
  (`playerState_t` 1552, `usercmd_t` 28, `saberInfo_t` 2156, `gameState_t`).
  Consequence: if the Rust modules are ever loaded under OpenJK, game-private
  layout parity vs the oracle is fine (those cross the seam only as opaque
  `sizeof`), but any struct the *engine* dereferences must be checked against
  the actual host engine — this tool now does that per-tree.

## v3 fixes (SP gentity_t investigation)

- **Backslash includes broke SP parses silently**: SP `q_shared.h:1370` does
  `#include "..\game\ghoul2_shared.h"` (Windows path), which fails on POSIX →
  every Ghoul2-dependent field errored out → clang kept `gentity_s` as an
  *empty* definition (1B, 0 fields) with no loud failure. Fixed generically:
  the 9 offending oracle headers are shadowed via libclang `unsaved_files`
  with slashes normalized (on-disk oracle untouched).
- Tree now fully expands at every occurrence (no dedup) — safe since by-value
  cycles are impossible in C.
- Anonymous unions render as `(anon union file:line)` via `is_anonymous()`
  (newer libclang gives them long non-empty spellings).
- **Lesson for absorption:** an empty-but-"successful" record definition is a
  failure mode; the real tool should hard-warn when a requested type has 0
  fields but a definition, or when its size is 1B in C++ mode.

## v4 — function call trees (`fn:Name --tree`)

- `fn:Name --tree` now prints a recursive call tree. Callee bodies live across
  the module's .c files, so this mode switches to a **unity parse** (one TU
  `#include`-ing every file in the module's `srcglob`); ~1s for all of MP game.
- Call graphs have cycles + heavy fan-in, so unlike structs each function
  expands once (`↺ expanded above`); repeated calls in one body collapse to ×N.
- Filtered: `__builtin_*` fortify-macro noise; out-of-tree functions collapse
  to `name  [libc/SDK]` leaves. Caveat: fortify rewrites `strcpy`→builtin on
  macOS, so plain `strcpy` calls vanish from the tree.
- Trees bottom out exactly at the ABI seam (`trap_*` in `g_syscalls.c`) —
  directly useful for slice planning. `player_die` full tree = ~2,400 nodes.
- Indirect calls through function pointers are not resolved (no referenced
  FUNCTION_DECL) — Raven's think/touch/die dispatch won't appear as edges.

## v5 — verified badges (size-assert cross-check)

- Ported status was already grepped from `crates/**/*.rs` (never the markdown),
  but ☑ was a name-match only. Now a struct/union/class badge is **verified**:
  ☑ requires a `size_of::<X>() == N` assert in the *same file* as the
  declaration, agreeing with clang's ground truth. Otherwise:
  `◐ declared, NO SIZE ASSERT (stub?)` or `✗ SIZE MISMATCH rust asserts […],
  oracle says NB`. Enums/aliases stay name-match (house style doesn't assert
  them). Same-file scoping + tag/typedef alias lookup were both necessary
  (MP's `gentity_t` assert must not vouch for SP's stub; asserts are written
  against `gclient_t`, clang reports `gclient_s`).
- Catches real things immediately:
  - SP `gentity_t` stub → flagged (previously badged ☑).
  - OpenJK runs: `clientPersistant_t`/`clientSession_t`/`gentity_t` flagged
    ✗ SIZE MISMATCH vs the oracle-derived Rust asserts — correct, OpenJK
    diverged.
  - **Repo finding:** 42 `#[repr(C)]` files carry no size assert at all,
    including full ports of heavy ABI structs (MP+SP `player_state.rs`,
    `entity_state.rs`, `usercmd.rs`, `trace_t.rs`, `collision.rs`,
    `trajectory.rs` — mostly pre-Wave files migrated from the old `src/`
    monolith). CLAUDE.md's "every ABI-crossing struct carries asserts" is
    currently aspirational for those. `--asserts` can generate the blocks.

## v6 — port packets (`portpacket.py`)

**Question:** can one invocation give a porting agent everything it needs to
port a function with NO file access? **Yes.**

`portpacket.py <module> <FunctionName> [--json] [--helper-cap N]` emits a
strict-markdown packet: signature + exact extent cite (from `cursor.extent`),
Raven comment, ready-to-paste house-style Rust doc header, verbatim
line-numbered body, callees classified [syscall | in-module | libc/SDK],
in-module helpers ≤ N lines inlined with their own cites, type closure with
verified badges, globals referenced (with static/extern + cite — feeds the
state-threading rule), the `trap_*` syscall surface, and paste-ready
`//TODO: Port` markers for every unported dep. `--json` for machine use.

- Verified: MP `DeathmatchScoreboardMessage` (body matches g_cmds.c:25-88
  exactly), MP/SP `G_Spawn` (helpers G_Error/G_InitGentity inlined; globals
  level/g_entities/SP `globals: game_export_t`), JSON round-trips.
- **Lesson:** clang's `raw_comment` (even with `-fparse-all-comments`)
  mis-associates comment blocks across neighboring decls (`va` got
  `BigShort`'s comment). Fixed by reading the comment block directly from the
  source lines above the extent (`preceding_comment`).
- closure.py refactor: badge logic extracted to `make_badger` /
  `build_alias_map` so both tools share verified badges. No behavior change.

## v7 — workflow-driven assert backfill (first real multi-agent run)

`.claude/workflows/port-assert-backfill.js` ran over all 42 assert-less
`#[repr(C)]` files (3-file smoke + 39-file full run; 40 Sonnet agents, ~8.5
min, ~1.1M subagent tokens). Results:

- **48 types asserted** across both trees, all from clang ground truth via
  `closure.py --asserts`; cargo workspace green; badges flipped to ☑.
- **1 genuine latent layout bug caught:** SP `gitem_t` was ported without the
  `#ifdef _IMMERSION` fields `pickup_force`/`forces` — but the SP game vcproj
  defines `_IMMERSION` in every configuration, so the real layout is 104 B,
  not 88. Fixed (fields added + asserts); badge now ☑ 104B.
- 5 other "mismatches" are documented deliberate stubs (SP `gentity_t`,
  SP `playerState_t`, `uiClientState_t`, `refdef_t`, `SpGameImportTable`) —
  correctly reported, correctly left alone. Bonus ground truth captured:
  SP `playerState_s` = 4992 B, `game_import_t` = 1048 B (~150 fn ptrs).
- Ops lessons: Workflow `args` can arrive as a JSON string (parse
  defensively); the named-workflow registry can serve a stale cached script —
  invoke by `scriptPath` when iterating.

## v8 — Wave 2 via port-wave v2 (packet-fed, pipelined)

Full bg wave: **45 MP + 11 SP types** (incl. 8 heavies: `vehicleInfo_t` 952B
w/ 137 anchors, `Vehicle_t` 976B, `bgLoadedEvents_t` 19272B, both `pmove_t`s)
in **37 min, 22 agents, ~990k tokens** — vs v1's 13 min / 8 agents / 293k for
just 3 types. Verify: cargo green, every struct badge ☑ (independently
re-swept). Raven comments preserved (e.g. vehicleInfo_t's vehFields warning).

Speedups that did it: sweep.py replaced scout agents (45 types in 0.8s);
packet-fed porters transcribe instead of exploring (manifest file pattern —
args carry ~9KB metadata, packets stay on disk); per-folder MP→SP pipelines
(no global barrier); haiku on trivial batches.

Cheap-model lessons (both bit this run, fixed in prompts):
- Packet-fed porters never see neighboring files, so the prompt must carry
  EVERY convention — missing file-level `#![allow(...)]` produced ~29
  warnings (Sonnet mediums too, not just haiku). Now explicit + zero-warning
  rule; post-run cleanup normalized 24 files.
- Agents read tool source (sweep.py) before running it — tools must be
  declared black boxes with sample output shown.
- Ops: zsh doesn't word-split unquoted vars — a batch commit script silently
  mispackaged 4 commits (caught, reset --soft, redone).

## If absorbed into the real workflow

- Generate assert blocks (`--asserts`) during ports instead of hand-deriving —
  this also fixes the "asserts are self-referential" gap.
- Add `-target i686-pc-windows-msvc` variant if a 32-bit `jampgamex86.dll`
  target is ever decided.
- Wire into the port-types skill as the scout step (closure before porting).

## v9 — Wave 4: three concurrent port-wave runs (modules + seams)

Full module wave: **236 types** (ui 44, cgame 62, game 130) in three
CONCURRENT workflow runs — **150 agents, ~55 min wall, ~5.5M subagent
tokens**, zero agent errors, every struct badge ☑ against clang, zero new
warnings. Landed the SP entity data model (`gentity_s` 1496B un-stubbing the
qshared opaque, `gclient_t` 7384B, `level_locals_t` 620536B), SP vehicles
(previously deferred), MP bot AI, both cgame/ui module states (`uiInfo_t`
342KB, `cg_t` 295/321KB), and the abi seam tables (`game_import_t`,
`uiimport_t`, `snapshot_t/_s`).

New machinery this wave (all committed):
- **Per-entry `{crate, srcDir, file}` overrides** in port-wave.js — one run
  mixes module-private targets with abi/qshared seam targets parsed from the
  same TU. `skipDocs` lets concurrent runs skip the shared-doc phase.
- **Multi-entry module TUs** (profile `entry` is now a list) — ai_main.h,
  wp_saber.h, G_Vehicles.h, bg_local.h, cg_media.h etc. were invisible to the
  single-entry parse; ~70 types would have been silently skipped.
- **Tree- then crate-scoped badges** — SP types were falsely ☑ via same-named
  MP files (`gclient_s`!), and mp_ui's `lerpFrame_t`(56) was mismatch-flagged
  against mp_cgame's (80). Badges now scan only the module's mode tree and
  prefer its own crate dir; a kept cross-tier `//TODO` under the tag name no
  longer shadows the port declared under the typedef name.
- **Case-insensitive owning-header match** — the oracle includes
  `G_Vehicles.h` (on-disk `g_vehicles.h`); macOS resolves it, libclang records
  it as spelled, and the SP vehicle types vanished from the sweep.

Lessons → fixes:
- **Mediums embedding sibling heavies fought red asserts** (Port phase runs
  before Heavy): the menu-widget family blobbed `menucommon_s` as `[u64; 11]`
  + TODO, roughly doubling those agents' tool calls, and a post-run reconcile
  pass had to swap blobs back to real embeds. Fixed for next wave: skeleton
  placeholders are now ABI-SIZED (`#[repr(C, align(A))] struct X([u8; N])` /
  `type X = c_int`), so any port order compiles and asserts correctly.
- Concurrent runs sharing crates (mp_abi cgame/ vs ui/) contended only on the
  cargo target lock — no cross-run fixer confusion observed; per-subfolder
  mod.rs ownership kept skeletons collision-free.
- `bot_settings_s` is owned by g_local.h, NOT botlib.h (checked assumption
  mid-run; the type was already ported in `level/` and the porter correctly
  turned its placeholder into a re-export).
- Cross-tier by-value deps surfaced a real pattern: SP `gentity_s` embeds
  game-tier enums (`material_t` et al) — kept as documented ABI-identical
  `c_int` aliases + TODO markers, mirroring MP's `gentity_t.client` decision.

## v10 — Wave 5: engine core via four concurrent runs

Numbers: 263 types (160 engine, 65 botlib, 19 ghoul2, 14 icarus, 5 rmg
hand-ported), 139 workflow agents, ~4.75 M subagent tokens, zero agent
errors, all four runs cargo-green on first verify. Wall: icarus ~7 min,
botlib ~14 min, ghoul2 ~11 min, engine ~24 min (the long pole, 75 agents).
v4's ABI-sized skeleton placeholders held up: no medium-embeds-heavy churn.

New machinery this wave:
- Multi-entry engine TU profiles (qcommon+cm+files+vm+server in one parse);
  new ghoul2/icarus/rmg profiles; botlib TU reproduces Q3's include order.
- sweep.py: comma-separated --header lists (one parse, N inventories);
  `cxx` flag on method/ctor/base-bearing records — drove the faithful-vs-
  C++-track split (124 of 377 swept types deferred); parse errors surfaced
  loudly on stderr.
- scan_ported: engine subcrates rank as own-crate; `pub use ... as Y`
  renames count as declarations (CCollisionRecord alias pattern).

Lessons → fixes:
- Include-order layout corruption is silent and real: `aas_entity_s` swept
  1 B (its `aas_entityinfo_t` field dropped — definition header parsed
  later), `interface_export_s` swept 1 B (missing `g_public.h`). Both
  caught pre-launch by the new stderr error surfacing + tiny-struct scan
  of manifests. Rule: every profile's entry list mirrors a real Raven
  compile-unit include order; sizes ≤4 B in a manifest are a stop signal.
- Cross-compiler layout caveat, now explicit: clang-mac IS the ground
  truth convention (all asserts ever written here). Win32-only fields
  (`timing_c`'s rdtsc stamps) compile out; std:: members mean C++ track.
- OpenJK's CMakeLists source_groups are a free placement oracle: their
  engine common/botlib/ghoul2/icarus/server groups map 1:1 to our crates,
  their botlib file list flagged the game-side `be_*.h` definition headers,
  and their dropped-RMG decision backed our rmg deferral.
- Porter rule gap: rules allow pointer-only deps to stay opaque, so in-wave
  siblings behind pointers (`indent_t.script`, `bstream_t.stream`, ...)
  came back `*mut c_void` + TODO and needed a manual reconcile pass. Next
  wave: extend RULES — pointer fields whose target is IN THE MANIFEST must
  use the real sibling type (placeholders make it safe already).

## v11 — Waves 6+7: client + renderer via two concurrent runs

Numbers: 258 workflow types (client MP 40 / SP 38; renderer MP 92 / SP 88)
+ 3 hand-ported (`soundChannel_t` — a Wave-1 gap the client sweep exposed:
`game/channels.h` is included by SP `q_shared.h` — and MP/SP `stereoFrame_t`),
126 workflow agents, ~4.4 M subagent tokens, zero agent errors, both runs
cargo-green on first verify. Wall: client ~21 min (54 agents), renderer
~28 min (72 agents), run concurrently.

New machinery this wave:
- mp/sp-client TU profiles (client.h + snd chain + FX headers + mp3struct);
  renderer profiles went multi-entry (tr_local + font/quicksprite/
  WorldEffects/landscape + cm_landscape.h for HEIGHT_RESOLUTION).
- glshim/: parse-only GL scalar-typedef headers (gl.h, GL/gl.h, MesaGL/gl.h,
  empty qgl_linked.h for MP) — qgl.h/glext.h parse without any GL SDK.
- Per-profile `flags`: `-fdeclspec` (shaderCommands_t's
  `__declspec(align(16))` typedef), `-fno-operator-names` (SP tr_local names
  fields `or` — clang was SILENTLY DROPPING them from viewParms_t/trGlobals_t;
  MSVC treats `or` as an identifier). Windows-type defines (HDC/HGLRC/BOOL/
  DECLARE_HANDLE(x)) fix qgl.h's unguarded WGL pbuffer section.
- _SHIM_PATCHES in backslash_include_shims: snd_local.h's eax includes
  (windows COM) drop out at parse time — verified no swept type embeds EAX.

Lessons -> fixes:
- The `or`-field drop is the nastiest silent-corruption class yet: no parse
  error mentions the struct, the field just vanishes. Caught by walking the
  FULL deduped diagnostic list per TU (not the first 3) and reading every
  "expected member name" site. Rule: any error whose location is inside a
  swept struct's line range is a stop signal.
- Error-recovery is layout-safe for default-args on fn-ptr members (MSVC
  extension, C++ forbids): probed refexport_t/uiimport_t offsets around the
  errored members — clang keeps the field, drops the default. Benign class.
- Vendored-by-value: channel_t embeds MP3STREAM (26656 B) by value, so
  mp3struct.h/small_header.h structs are ported as layout types even though
  mp3code stays replaced. "Vendored -> skip" applies to CODE, not to layout
  the faithful port depends on.
- v10's porter-rule gap recurred (skyParms.outerbox, landscape shader_t,
  bmodel.firstSurface, srfTerrain.landscape came back `*mut c_void` + TODO):
  RULES now says in-manifest/below-tier pointees must use the real type;
  opaque + TODO is only for C++-track / platform / higher-tier targets.
