# Gate-claim sweep (`#ifdef` activation audit) — 2026-07-08

Task: verify every conditional-compilation region in the jampgame TU set against
the build that A/B parity is actually measured against, catching both lying
rationale comments and silent drops. Motivated by the `g_log.c` incident
(`fix(game)` fe38963d): ~16 functions stubbed as dead code under a false
"`LOGGING_WEAPONS` is never `#define`d" claim — the define is at `g_log.c:3`.

## Method

`tools/closure-prototype/gatesweep.py`. Ground truth = the referee oracle
dylib's preprocessor state (`tools/referee-oracle/build.sh`: real GCC,
`-std=gnu++98`, `QAGAME _JK2MP __linux__ _FORTIFY_SOURCE=0 NDEBUG`). Per-region
verdicts via sentinel-token injection + real preprocessing, so same-TU
`#define`s (the g_log trap), header defines, and flags resolve by construction.
Joins: (a) Rust gate-rationale comments → TRUE/LYING; (b) ACTIVE-region symbols
missing from Rust → silent drops; INACTIVE-region symbols present → ported dead
code; (c) config-flip probing for retail toggles (`FINAL_BUILD`, `_DEBUG`,
`MISSIONPACK`, …) — flagged, never silently resolved.

Note: `closure.py`'s jampgame parse profile uses `MISSIONPACK + _JK2`; the
referee (authoritative) uses `_JK2MP` without `MISSIONPACK`.

The script itself was audited (main-session review, 2026-07-08): all 89 TUs
preprocess cleanly (now asserted in-script), no backslash-continued directives
exist in the corpus (injection now handles them anyway), the UNMATCHED claim
bucket was hand-checked for missed lies (none), and symbol extraction was
spot-verified against `g_log`'s gated symbol set (10/10 captured).

## Results

- 89 TUs, 1,089 conditional regions: 536 ACTIVE / 553 INACTIVE.
- ~200 rationale claims: **6 LYING** at sweep time, 0 silent drops, 0 ported
  dead code.
- Config-flip gates: 292, dominated by fixed build-identity toggles (`_XBOX`,
  `_JK2MP`); retail-relevant (`FINAL_BUILD` 57, `_DEBUG`/`NDEBUG` 37) are
  enumerated in the sweep output, none actioned (referee state is the parity
  target).

## Fixes landed (serial wave, each verified: workspace build, zero-warning
all-targets check, full `mp_game` suite, both A/B corpora at zero divergence)

1. **13cc9a50** `BOT_STRAFE_AVOIDANCE` (behavioral) — `ai_main.c:1548` defines
   it unconditionally; the gated call block at `ai_main.c:7243-7257` was
   dropped as "(undefined)". `BotTrace_Strafe` was fully ported but had zero
   call sites: bots never strafe-avoided obstacles. Call block ported.
2. **2647ad28** `DEBUG_SABER_BOX` (active debug) — defined at `g_local.h:82`;
   `G_DebugBoxLines` was a 3-line stub and three gated call sites were
   dropped. All ported. Runtime-gated by `g_saberDebugBox` (`CVAR_CHEAT`,
   default 0), which is why corpora never exposed the asymmetry.
3. **4147e665** `FINAL_BUILD`-family (4 sites) — `#ifndef` debug blocks live in
   the referee build: `G_SpewEntList` (had also dropped always-compiled
   `Com_Printf` lines and the `entspew.txt` write), two `NPC_combat.c`
   combat-point map-error prints, and a `g_saberDebugPrint`-gated attack-power
   print. All ported with their runtime gates; per-site reasoning confirmed
   none were reachable in the corpora (no masked divergence).
4. Truncation-heuristic adjudication — 6 remaining hits all **FAITHFUL** (no
   change): the three `AnimateVehicle` flags were a tooling artifact
   (identically-named per-vehicle-class oracle symbols matched against
   `SpeederNPC.rs`'s correctly-empty stub; real ports are full-length in
   `AnimalNPC.rs`/`FighterNPC.rs`/`WalkerNPC.rs`), and
   `PM_IsRocketTrooper`/`SpeederNPC Update`/`AnimalNPC DeathUpdate` match
   Raven's genuinely tiny MP-compiled bodies.

Post-fix re-run reproduces all region counts; the 2 residual "LYING" hits are
classifier false-positives on the corrected comments (mixed-polarity multi-macro
blocks — one comment truthfully calls one macro dead and another live; the
block-level polarity heuristic cannot attribute per-macro). Verified accurate by
hand.

## Standing limitations (also in the sweep's own output)

- Call-site drops inside ACTIVE gates are caught only via comments (the
  BOT_STRAFE shape); an uncommented call-site drop is a blind spot.
- Body-level truncation is covered only by an advisory line-count heuristic —
  full body verification remains the per-file oracle review / referee-swap
  campaign on the roadmap.
- The truncation heuristic matches symbols crate-wide and can cross-file
  collide on identically-named functions (see `AnimateVehicle` above).
- Claims audit scope: `crates/mp/game/src` only — rerun when engine/cgame gain
  ported logic.
