# Engine lockstep referee — plan (SETTLED 2026-07-13)

User directive (2026-07-13, halting the statistical bot A/B): *"Make the
referee in the engine work so we can run jampgame from Raven AND run our
jampgame in unison, play frame-by-frame and see the differences."*

Why now: every bug of the last two days lived in referee-blind code (bot/NPC
paths, engine-island seams, network encoding). The in-repo mock referee cannot
execute bot brains (botlib is engine-side; the mock replaces the engine), and
the statistical A/B (kill-rate/MOD histograms vs OpenJK) measures configs as
much as code. This rig makes live bot combat — and live human play —
byte-comparable against Raven's real module, frame by frame.

Supersedes the parked `docs/plans/2026-07-07-rust-referee.md` where they
conflict; inherits its diff/report vocabulary and the §F proof discipline.
The existing six-scenario mock referee STAYS as the per-commit gate.

## Settled rulings (user, 2026-07-13)

1. **Architecture: two lockstepped engines.** Two `mp_app`-based engine
   processes — the PRIMARY hosts our `jampgame` cdylib, live bots, and the
   human player; the SECONDARY hosts Raven's oracle `jampgame` dylib. A
   driver steps them in lockstep. (Shadow-module-in-one-process was
   considered and rejected.)
2. **Divergence policy: cvar-switched.** `ref_haltOnDiverge 1` freezes both
   engines into step mode (`ref_step` = advance one frame, `ref_diff` =
   field-level delta); `0` logs the divergence and resyncs the secondary
   from the primary's snapshot, play continues.
3. **Diff depth: states + syscall digest.** Per-frame byte-diff of every
   entityState/playerState pair PLUS the ordered per-frame syscall digest of
   each module (the proven comparison from `crates/jampgame/tests/referee.rs`,
   applied live).
4. **Sequencing: this rig is built FIRST; the held safe-state campaign chain
   pushes only after it validates live bot combat** (goal G7). The push hold
   (2026-07-13) remains in force until then.

## Design keystones (from the settled architecture)

- **Single-source inputs.** Nondeterminism must run ONCE. Botlib runs only in
  the primary; the primary taps every client's `usercmd_t` per frame (human +
  bot alike) and the driver mirrors them into the secondary, whose clients
  are synthetic replicas (`bot_enable 0` there). The human plays on the
  primary; their inputs mirror like any other client's.
- **Engine-side RNG must be pinned.** The game modules' parity-critical LCG
  lives in-world (`bg_state.rng` — deterministic given identical call
  sequences). The ENGINE's own rand services (`flrand`/`irand` host calls)
  must be seed-pinned identically in both engines at boot; the syscall digest
  verifies the call sequences stay aligned.
- **Oracle module on 64-bit needs a width patch.** Raven's `vmMain` takes C
  `int` args; engine→game pointer-carrying calls (GAME_NAV_* callbacks — bots
  hit these constantly) truncate on LP64 (proven crash, 2026-07-13). The fix
  belongs in `tools/referee-oracle/build.sh`'s throwaway-copy patch layer
  (precedent: the SnapVector/rint retail-win32 patches). The oracle source
  tree is NEVER edited.
- **Frame protocol.** Driver: step primary one server frame → collect
  {usercmds issued, snapshots, syscall digest} → feed mirrored usercmds →
  step secondary one frame → collect → diff → report/halt/resync per cvar.
  `sv_fps 20`, identical launch cvars both sides, map/gametype pinned.
- **Resync semantics (log-and-continue mode).** Secondary's world is
  overwritten from the primary's authoritative snapshot after a divergence is
  logged, so one divergence doesn't cascade into noise. Resync writes through
  the module's LocateGameData-registered memory.

## Goals (ordered; each ends referee-green on the existing gates + its own done-criterion)

- **G1 — Oracle module survives 64-bit bot hosting.** Patch
  `tools/referee-oracle/build.sh` (throwaway copy): widen `vmMain` arg words
  (and its dispatch casts) `int` → `intptr_t`. DONE WHEN: our engine boots
  `mp/ffa3` with 8 bots against the PATCHED oracle dylib as its game module
  and runs 10+ minutes without truncation crashes (the exact scenario that
  segfaulted pre-patch).
  **STATUS: DONE 2026-07-13.** Two fixes: (a) build.sh widens vmMain params +
  return + the `(int)ClientConnect` denied-string cast to `__INTPTR_TYPE__`;
  (b) the patched dylib immediately exposed an engine-side twin —
  `BOTLIB_USER_COMMAND`'s clientNum read full-width in `sv_game.rs`, but a C
  module's `int` varargs leave the slot's high 32 bits garbage (our Rust
  module always passes clean full words, masking it). Full dispatch audited:
  only bad site. Six mock scenarios byte-identical on the patched dylib;
  11-min 8-bot soak on mp/ffa3 clean.
- **G2 — Input tap + synthetic-client injection in our engine.** Server-side:
  per-frame capture of every client's `usercmd_t` (+ connect/disconnect/
  userinfo events) to a channel/log; injection path that feeds captured
  usercmds into `SV_ClientThink` for synthetic replica clients. Study the
  live-hunt-era `sv_recordInputs`/`sv_replayInputs` failure first — the
  attack-button loss was in record/replay FRAMING (parked-plan finding);
  root-cause it so the injection path is sound. DONE WHEN: a single engine
  can replay its own captured session with byte-identical outcomes.
- **G3 — Lockstep driver.** New tool (suggest `tools/lockstep-referee/` or a
  `crates/jampgame/tests/` binary target): boots both engines (primary: our
  module + bots + open player slot; secondary: patched oracle module,
  synthetic clients), owns the frame-step protocol and input mirroring, pins
  engine RNG seeds and launch cvars. DONE WHEN: both engines step 1000+
  frames in lockstep on `mp/ffa3` with bots, inputs mirrored, without
  protocol stalls.
- **G4 — Per-frame comparison.** Wire the referee's existing diff machinery
  (entityState/playerState byte-diff + first-divergent-field naming + ordered
  syscall digest) into the driver's per-frame loop. DONE WHEN: an
  artificially injected divergence (e.g. a one-off cvar tweak on the
  secondary) is detected on the exact frame it occurs, with field-level
  attribution.
- **G5 — Divergence UX.** `ref_haltOnDiverge` cvar (halt→step mode with
  `ref_step`/`ref_diff` console commands; log→resync path per the resync
  keystone). DONE WHEN: both modes demonstrably work mid-session.
- **G6 — First real hunt.** Run bot combat (`mp/ffa3`, 8 bots, skill 4,
  retail cvars) to first divergence; triage it fully (frame, entity, field,
  oracle-vs-rust code path). The bot force-power/saber MOD-histogram skew
  (rust bots: 75% MOD_FORCE_DARK; reference: 86% MOD_SABER — measured
  2026-07-13 pre-pin; root cause unknown, g_maxForceRank config difference
  eliminated as sole cause only after pinning) is the expected first catch.
  DONE WHEN: first divergence root-caused, fixed, and the fix
  referee-verified.
- **G7 — Soak to parity.** Iterate G6 until one hour of continuous bot combat
  PLUS a live human play session (the user, frame-mirrored) complete
  byte-identical. This is the Stage-R bar from
  `docs/roadmap-final-stages.md` ("an hour of recorded dueling replaying
  byte-identical ≈ proof"), achieved live instead of via tapes.
- **G8 — Land and release the held chain.** Gates + docs (this plan's status
  updates, roadmap Stage-R status, memory notes), fold in the safe-state
  campaign wrap-up (plan §4 finals, roadmap "safe-state COMPLETE"), THEN push
  the entire held commit chain (the push hold lifts here, per ruling 4).

## Standing constraints

- The oracle tree (`oracle/**`) is never edited; all oracle-side fixes are
  `build.sh` throwaway-copy patches.
- Every commit in this build stays green on the EXISTING gates (346 workspace
  tests + six-scenario mock referee + fmt + i686 lane) — the new rig is
  additive, not a replacement.
- Worker protocol as in the safe-state campaign: worktree agents
  (Opus/Sonnet), thin gates per worker, serial human-reviewed integration,
  one commit per goal or sub-goal, push held throughout.
- Live-server rig rules: kill by PID only, never by name; announce
  user-facing server readiness via `say`.

## State at plan time (2026-07-13)

- Held local chain: safe-state Stage 2a + 2b (12 shards) + Stage 3 + pass-A
  fixes (`c3aa37d8`) — all referee/tests-green, unpushed on master atop
  `2603f697…926cccb5` (pushed pre-hold).
- Verification pass A closed (5 hoist-order divergences found and fixed).
- Verification pass B halted mid-measurement by this directive; its one
  complete pre-pin sample produced the MOD-histogram skew above.
- Servers all stopped. Oracle dylib currently UNPATCHED for 64-bit vmMain.
