# ui port plan (full process; transcription starts 2026-07-25)

Companion: `scoping.md` (this folder) (surveys + frozen-vs-
free census — the design ground truth). Ruling 2026-07-24: port **directly to
the idiomatic shape** (blind-faithful pass retired), **minimal deferrals**,
every deferral/port-note concretely greppable. Target: the EXISTING `mp_ui` +
`mp_uishared` crates building a ui dylib that loads under openjk.app as a
drop-in, validated by menu-parse goldens + live use.

## Marker law (extends porting-rules.md)

- `//TODO: Port <subject>` + `// Source:` — an *unported dependency* hit
  during transcription. Wave ordering exists to keep this near zero; every
  occurrence is a wave-planning defect fed back into the manifest.
- `// DEFERRED: <subject> — <reason> (<ruling/stage cite>)` + `// Source:` —
  a *deliberate* deferral. Only legal with a cite; no cite → review rejects.
- `// PORT-NOTE: <note>` — translation notes (dictionary applications, §19
  UB picks, dead-surface drops).
- Wave gate: marker census (`grep -rn "TODO: Port\|DEFERRED:\|PORT-NOTE:"
  crates/mp/ui crates/mp/menu`) recorded per wave; a wave that ADDS
  `TODO: Port` fails review. Track closes at zero `TODO: Port`, only cited
  DEFERRED.

## Translation dictionary (applied during transcription)

qboolean→`bool`; `char*`/`char[N]`→`&str`/`String` (Class C only — census-
gated; Latin-1 discipline at byte seams); out-params→returns; file-scope
statics→`UiWorld`/subsystem fields (no `static mut`, no ambient cells — the
G_SoundIndex lesson is day-one law); intrusive pools→arena + index handles;
C fn-ptr tables→`match` dispatch (trait only where the set is open —
`DisplayContext`); `#define`→`const`/enum with enum-vs-alias fidelity; one
type per file; doc comment + `Source:` cite per item; imports at top, no
inline qualified paths; Raven names kept.

## Minimal-deferral strategy

Dependency-first, not file-order:

1. One libclang parse per TU (closure-prototype walker; validated 2026-07-24:
   ui_main.c parses with `-x c -DUI_EXPORTS`, 208 fn defs) → symbol
   inventory + call graph → `ui-fn-manifest.json`.
2. Types before functions (sweep.py packets: verbatim slice + assert block).
3. Functions in **topological waves**: leaves first; a fn enters a wave only
   when every callee/type/world-field it needs exists; mutual-recursion
   cycles ship as one atomic wave.
4. Every packet carries a threading digest (UiContext fields, trap wrappers,
   callback-trait methods touched) + the dictionary — blind transcribers
   never invent shape.
5. Mechanical referee per wave: `cargo check`, banned-pattern greps, marker
   census delta.

## Stages

- **U0 — tooling prep.** The `mp-ui` profile ALREADY EXISTS
  (`closure.py:56-60`, lang=c, `UI_EXPORTS`, ui+game includes; builtin
  include dir handled generically at `closure.py:413`) — confirm/extend it;
  the genuine work is the ui packets generator. Emit the
  fn manifest, call graph, wave partition; type inventory from
  `ui_local.h`/`ui_shared.h`/`menudef.h`/`ui_public.h`/`ui_force.h`. Build a
  ui packets generator on the packets3 pattern (digests reference UiWorld/
  UiContext/DisplayContext instead of GameContext).
- **U1 — crate audit + frozen layer (NOT greenfield — review finding).**
  `crates/mp/ui` (`mp_ui`, cdylib+rlib) and `crates/mp/uishared`
  (`mp_uishared` — the MenuSystem type layer: menu_def_t/item_def_s/
  display_context_def_t etc., ALREADY a dependency of both `mp_cgame` and
  `mp_ui`) exist from the type-port campaign, as does the ABI seam
  (`crates/mp/abi/src/ui/`: imports/exports/vmcalls + 151 syscall files)
  with `uiClientState_t` ported and `glconfig_t` in qshared. U1 = audit
  those stubs, reconcile the frozen set (drop already-ported items), and
  build only the missing trap-wrapper String/bool surfaces. `mp_uishared`
  EVOLVES into the idiomatic MenuSystem (no new `mp_menu` crate). Dead
  surface dropped with PORT-NOTEs: `uiStatic_t uis`, dead traps per census.
  ui has ZERO Class-A shared memory — no choke-point machinery needed.
- **U2 — root-type sit-down (user ratifies before any transcription).**
  `UiWorld`: uiInfo_t spine + ui_force.c globals + per-file statics folded
  in (gameinfo arena/bot caches, saber parse state, preview timers, connect
  latches, siege globals). `MenuSystem` owned by composition (menuDef/
  itemDef arena + indices replacing the raw-pointer graph; `String_Alloc`
  intern pool → owned String table; open-menu stack as indices).
  `DisplayContext`: decide REPLACE-vs-WRAP — `mp_uishared` already carries
  a faithful `#[repr(C)]` fn-pointer `display_context_def_t` with offset
  asserts and two dependents; the idiomatic trait either supersedes it
  (asserts retired by ruling) or wraps it (sit-down decision).
  `UiContext` threading shape. The bg `UI_EXPORTS`/`WE_ARE_IN_THE_UI`
  callback story (GameCallbacks-pattern trait, own-state impls). The
  animation-cache duplication collapse: ui reuses mp_bg's animation module
  instead of porting Raven's hand-synced fork (PORT-NOTE at the site).
- **U3 — type port.** sweep packets over the ui headers; one type per file;
  asserts on every frozen struct; Class-C structs land already-idiomatic
  (String fields, bool, Options).
- **U4 — function port.** Topo waves per manifest; parallel blind
  transcribers per packet; wave gates. The 83+34 keyword→fn-ptr parse
  tables become `match` dispatch; the 166-case ownerdraw and feeder
  switches stay `match`es. `va()` call sites (210 in ui_main.c) become
  `format!` per the settled q_format patterns.
- **U5 — integration + goldens.** vmMain dispatch + dllEntry exports; dylib
  target builds. Menu-parse goldens (§F pattern): a `tools/ui-oracle/`
  harness compiles the unmodified oracle menu parser standalone — backed by
  a REAL precompiler, not stubs (review finding: `ui_shared.c` tokenizes
  via engine-side `trap_PC_*` = botlib l_precomp, 23 call sites, plus
  `trap_SP_*` string lookups) — link the oracle botlib precompiler in the
  dumper and route our side through the ported botlib precompiler. Dump the
  parsed menu tree for every shipped `.menu` asset; committed fixtures; our
  parser must reproduce them. Deterministic, no GL.
- **U6 — live gate.** OpenJK's UI ABI is DUAL (review finding,
  `cl_uiapi.cpp`): it searches basename `ui<ARCH><EXT>` (e.g.
  `uiarm64.dylib`), prefers the native `GetModuleAPI`/`GetUIAPI` arm, and
  falls back to legacy `vmMain`+`dllEntry`. Decide the arm at U2 (legacy
  vmMain matches our jampgame precedent; native is OpenJK-only). FIRST a
  stub-dylib load test under openjk.app with the correct basename (dlopen +
  chosen-arm handshake). Then the real
  module: main menu renders, server browser populates, player-model preview
  draws (the ~30 G2 traps), settings/binds work, in-person pass. If U5
  goldens leave behavior gaps, add a trap-stream referee (scripted
  key/mouse replays, oracle-vs-rust outbound trap diff).

## Sequencing & capacity

ui is the lead track. Renderer sit-downs (its plan's R0-R2) may run during
ui waves; cgame follows ui on this same process (adding the prediction-seam
design from the scoping doc). Wave size tuned so review (diff every worker,
banned-pattern greps) stays the bottleneck-free step it was in the #19
campaign.

## Plan validation record (2026-07-24)

- Walker venv + libclang alive; ui TU dry-run parse: 208 fn defs, only the
  clang-builtin include dir missing (profiles already solve this).
- Surveys + engine-retention census complete (scoping doc).
- Adversarial review run 2026-07-24; findings folded in.
