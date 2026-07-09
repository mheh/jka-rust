# Handoff: constant prediction-miss investigation (2026-07-07)

Status: CLOSED (2026-07-08) — see the closure note at the end of this doc.

Resume point for the client-prediction bug. Read this plus `docs/audits/clientthink-parity-audit-2026-07-07.md` (the prior session's audit) and you have the full state. This session was **investigation only** — no code, test, or doc changes besides this file.

## Symptom

taystjk client connected to the Rust `jampgame` server mispredicts constantly; the camera hits invisible boundaries that don't exist. Happens both **standing still and moving**. The two Jul 6 client crashes (`BG_SetAnim` via `CG_PredictPlayerState`) are already fixed — only the miss remains.

**Client log evidence** (`~/Library/Application Support/Steam/steamapps/common/Jedi Academy/SWJKJA.app/Contents/taystjk/qconsole.log` — note: truncated on every client launch; the surviving session is Jul 7 01:13–01:17 with `cg_showMiss 1`):
- 291 `prediction error` / 264 `Prediction miss` lines in ~20 s.
- Miss magnitudes tightly quantized: `17.884399` ×54, `18.000000` ×27, `17.897583` ×25, `17.910034` ×22, `17.921631` ×11 — plus occasional small legitimate values (0.11–0.60).
- `PredictionTeleport` once at spawn. No parse/delta/snapshot warnings of any kind.

## Established facts (the arithmetic that anchors everything)

- The miss is a near-constant vector **(16, 8, 0..2) per 25 ms server frame** (also recorded in the prior audit).
  - Standing still: √(16²+8²) = **17.8885** → the observed hard floor.
  - With z=2: √(16²+8²+2²) = **18.0 exactly** → the observed ceiling cluster.
- Equivalent phantom velocity: **(640, 320, 80) u/s** — ratios 8:4:1. Looks like a map coordinate or a bit pattern, not a plausible movement velocity.
- Interpretation candidates: (a) snapshot `ps.velocity` carries this constant; or (b) the **server physically moves the player** (16,8,2)/frame while the client commands nothing — the latter also explains "camera hits invisible boundaries" (the server-side player drifts into *real* walls elsewhere).

## Hypotheses tested this session — all four REFUTED (static, vs vendored oracle)

1. **playerState layout / origin-data-into-velocity clobber — REFUTED.**
   Rust `playerState_t` (`crates/mp/qshared/src/common/mp/qcommon/player_state.rs:78-248`) is field-for-field identical to oracle `q_shared.h:2169+`; compile-time asserts pin `offset_of(velocity)==32`, `size==1552` (`player_state.rs:258-264`). `gclient_t.ps` asserted at offset 0 (`client/gclient.rs:235`); `LocateGameData` stride = `size_of::<gclient_t>()` over a contiguous `Box<[gclient_t; MAX_CLIENTS]>` (`g_init_game.rs:211-219`) — engine and game agree byte-for-byte. No write anywhere copies origin/spawn/trBase into `ps.velocity`; ClientSpawn zeroes the whole gclient then copies spawn_origin into `ps.origin` only (`g_client.rs:2848, 3239-3240`), matching C.

2. **STEPSIZE / ground-snap / viewheight Z divergence — REFUTED.**
   STEPSIZE 18 (`bg_slidemove.rs:37`), mins/maxs (-15,-15,-24)/(15,15,40) (`g_client.rs:126-129`), viewheights 36/12/-16 (`viewheight.rs`), spawn `origin[2] += 9` — all match oracle. Decisive: a step/ground bug would give a **vertical** (0,0,18) miss ≥18; the observed miss is **horizontal-dominant and sub-18** when standing. "17.884 ≈ STEPSIZE" was a coincidence — it's √(16²+8²).

3. **usercmd corruption on the async GAME_CLIENT_THINK path — REFUTED.**
   Dispatch (`world/game_context.rs:158` → `ClientThink(clientNum, NULL)` → `trap::GetUsercmd` into `pers.cmd`, `g_active.rs:3802-3833`) matches `g_main.c:526`/`g_active.c:3649+` exactly; no wrong-slot indexing; no double-integration (`G_RunClient` early-returns, `g_active.rs:3840`). `usercmd_t` layout asserted byte-identical, moves are correctly **i8** (`usercmd.rs:16-49`). `PM_CmdScale` (`bg_pmove.rs:1938-1963`) and the SHORT2ANGLE/ANGLE2SHORT shift math (`bg_pmove.rs:7982-8004, 2090`) are bit-faithful. Zero input provably yields zero wishspeed.

4. **Stale `pushVec` server-only drift — REFUTED.**
   `G_AddPushVecToUcmd` (`g_active.rs:1419-1465`) matches `g_active.c:1211-1243` exactly: early-return on zero pushVec, clear-guard `pushVecTime < level.time` (not inverted), `+2000` ms windows in the only set sites (NPC nav, `g_navnew.rs`, faithful). Zero-contribution for a human client. Widened pass: **no** divergent ucmd/`ps.velocity` mutation exists anywhere in the ClientThink_real→Pmove span (`g_active.rs:2196-3009` vs `g_active.c:2067-2895`), line-for-line.

## Remaining suspects (in rough priority order)

1. **The engine seam (out of repo).** Everything above proves the Rust module is self-consistent with the *vendored* oracle headers. If the **actual engine binary the server runs** has a different `playerState_t` netfield table / offsets than `oracle/oracle/codemp/game/q_shared.h`, the wire serializer reads the wrong bytes — invisible to every in-repo audit. Identify exactly which engine binary hosts the server and diff its netfields/headers against the vendored oracle.
2. **Runtime-only state divergence.** A `ps` field feeding Pmove (speed, gravity, pm_flags, groundEntityNum) that differs between the server's sim and what the client's prediction starts from — not findable statically; needs the probe below.
3. **The client side (taystjk cgame prediction setup)** — never examined. A constant miss can equally mean the client predictor diverges while the server is right.

## Recommended next steps (not executed — user paused here)

1. **Dynamic probe (~10 min, decisive between suspects 1 and 2):** temporarily log `ps.origin`, `ps.velocity`, `groundEntityNum` in `ClientEndFrame` for a standing client; connect taystjk with `cg_showpos 1` + `cg_showMiss 1`.
   - Server logs nonzero velocity / drifting origin → game-state bug server-side; bisect from there.
   - Server logs zero velocity + stable origin, but client still misses → engine serialization seam (suspect 1) or client-side (suspect 3).
2. **Pull/build an OpenJK dedicated server for macOS from source** (user's suggestion) and load the Rust `jampgame` module into it. This attacks the engine seam directly: the built engine's headers/netfields are *known* (same source tree), so if the miss reproduces there, the engine seam is exonerated and it's game-side/runtime; if it doesn't, the currently-used engine binary is the mismatch. While in the engine source, add logging at the serialization point (`MSG_WriteDeltaPlayerstate` / `SV_BuildClientSnapshot`) to dump the `ps.origin`/`ps.velocity` bytes actually read from the game's memory — this sees the seam itself, which no game-module probe can. Run with `sv_fps` set very low so per-frame deltas are large and the logs are humanly readable.
3. If engine seam confirmed: diff the old engine binary's playerstate netfields against the vendored headers to find the exact skewed field.
4. Then close the prior audit's referee gaps (they double as the offline reproducer): ground-returning mock `trap_Trace` (`crates/mp/game/tests/common/mod.rs:463-489`) and an async `GAME_CLIENT_THINK` scenario (currently only `g_synchronousClients 1` is exercised).

## Session bookkeeping

- Investigated repo: this one (`jka-rust`). The Claude session ran from `jedi-academy-rust` (wrong cwd) — resume in a fresh session **from this repo** and point it at this file.
- Working tree was clean at HEAD `ddc598be` throughout; nothing was modified except this handoff.

---

# SESSION 2 UPDATE (2026-07-07 evening) — root cause cornered: triggers never fire

Live A/B instrumentation (Rust module vs stock C module, same OpenJK arm64 engine built from `lmd-rewrite/OpenJK` @8cce3ea, same instrumented OpenJK client) produced a decisive chain:

## Confirmed findings
1. **The A/B**: stock C module = zero prediction misses; Rust module = misses that are LOCATION-dependent, concentrated at movers (elevators). Elevator on mp/ffa1 *disappears* when approached/touched (invisible or fall-through; server-side its state never changes).
2. **Wire + state exonerated**: playerState encode(SEAM)/decode(RSEAM) byte-perfect per commandTime; ps fields at snapshot (origin/vel/gnd/pmf/pm_type/basespeed/gravity/delta_angles/pm_time) correct; usercmd path, command timing, Pmove loop, pushVec, stepSlideFix, mover game code, areaportal calls, entityShared_t layout (all offsets asserted vs engine contract), G_ENTITIES_IN_BOX marshaling — ALL audited byte-faithful vs oracle.
3. **The smoking gun**: on the Rust module, **G_TouchTriggers never dispatches a single trigger touch** (probe `TOUCHDISPATCH` = 0 across full sessions), so `trigger_multiple → Use → Use_BinaryMover` never runs → **zero mover state transitions ever** (probes on SetMoverState/Use_BinaryMover_Go; C fires them on touch, ent=114 verified). Downstream: movers never move, areaportals never open, engine culls the mover from snapshots as the player crosses areas → invisible elevator + prediction errors near it.
4. Not the cause: frame loop healthy (`PROBE RUNFRAME`: 129 iterated, 10 movers reach G_RunMover every frame); doors are *targeted* (Think_MatchTeam — door self-triggers legitimately absent); triggers spawn correctly (`TRIG-INIT`: contents=0x400 = JA CONTENTS_TRIGGER, sane brush bounds, linked at spawn tail; ent=52 box (2247,767,1799)-(2353,897,1809) is the ffa1 elevator trigger); think dispatch enum complete; no panics (panic hook installed, silent unwind risk noted: exports are extern "C-unwind" with NO catch — a real latent hazard).

## Exact resume point (one round from the answer)
The working tree has all probes (uncommitted, marked `PROBE(seam-test)`), including the FINAL one not yet exercised with a player: in `G_TouchTriggers`, `PROBE TT num=<EntitiesInBox result> mins=... maxs=...` printed once per second. **Next session: start the server (`~/Developer/jka/seam-test/start-seam-test.sh`, Rust dylib already staged in base/), connect (`~/Developer/jka/seam-test/launch-client.sh`), stand INSIDE trigger ent=52's volume (near 2300,830,1800 on mp/ffa1) and read `PROBE TT num=`:**
- `num=0` while inside a linked trigger volume → the engine's box query doesn't see game-linked brush triggers → investigate the SetBrushModel→LinkEntity→absmin seam (engine computes absmin from r.currentOrigin+mins/maxs at link; brush triggers rely on bmodel bounds) and G_TouchTriggers' mins/maxs inputs (printed in the probe).
- `num>0` but TOUCHDISPATCH=0 → a per-candidate filter eats it (touch enum presence on the entity, contents mask, EntityContact call) — probe each filter branch next.

## Test rig (all reusable)
- Server dir `~/Developer/jka/seam-test/` (patched `openjkded.arm64` with SEAM logging; `start-seam-test.sh [map] [sv_fps]`; module dylibs: `base/jampgamearm64.dylib` = active, `jampgamearm64-RUST.dylib`/`-STOCK-C`/`-INSTR-C` staged; all round logs archived as `seam-server-round*.log`, `client-round*.log`).
- Instrumented OpenJK client: `launch-client.sh` (RSEAM/MISSVEC/RSTART/RSTEP probes; console tee'd to `client-console.log`).
- OpenJK worktree (patched engine+cgame+C-module builds): scratchpad `openjk-seam` git worktree of `~/Developer/Milo/lmd-rewrite/OpenJK` — remove later with `git -C ~/Developer/Milo/lmd-rewrite/OpenJK worktree remove --force <scratchpad>/openjk-seam`.
- jka-rust working tree: probe-instrumented (g_active.rs, g_mover.rs, g_main.rs, g_trigger.rs, g_utils.rs, trap.rs, jampgame/lib.rs) — all temporary, revert after diagnosis. HEAD ddc598be, nothing committed.
- Dual-debugger recipe (user requested): two lldb sessions (server: `lldb -- ./openjkded.arm64 …`, client: `lldb -- openjk.arm64 …`), raise `sv_timeout`/`cl_timeout` to 600, sync both sides with conditional breakpoints on the same `to->commandTime` in MSG_Write/ReadDeltaPlayerstate; `watchpoint set variable g_entities[N].nextthink` to catch state clobbers.

---

# RESOLVED (2026-07-07 late evening) — two root causes, both fixed and live-verified

## Bug 1: `GlobalUse` was a no-op stub
`crates/mp/game/src/g_utils.rs` `GlobalUse` carried a PORT-NOTE(fn-pointer-dispatch-no-ctx) and returned without dispatching. Every targeted activation in the game funnels through it → no trigger could ever fire its target → no mover/door/relay/usable ever worked. **Fix:** threaded `ctx` in, dispatch via `ent_fn_enums::dispatch_use`; 9 call sites updated (incl. func_usable pain/die which needed ctx threaded through their dispatch arms).

## Bug 2: niche-layout hazard — zeroed memory decodes as `Some(variant 0)`, defeating C NULL-handler semantics
The 7 `Option<EntXxx>` dispatch fields in `gentity_t` (think/reached/blocked/touch/use_/pain/die) have no reserved zero discriminant, and `Option`'s niche is the value AFTER the last variant. `write_bytes(0)` (C memset parity) therefore yields `Some(first_variant)` instead of `None`: zeroed `touch` == `Some(HolocronTouch)`, `think` == `Some(AimAtTarget)`, `blocked` == `Some(Blocked_Door)`, etc. Movers never set `touch`, so standing on any elevator ran **HolocronTouch on the door**: hid the entity like a picked-up holocron (the "disappearing elevator"), granted the player a phantom "Jedi Enlightenment" pickup (observed live!), and wrote the holocron respawn pair `(1.0, level.time+30000)` into the door's `pos2` — which, once Bug 1 was fixed and doors could move, sent the elevator flying to y≈70000 at 38k u/s.
**Fix (interim, semantics-preserving):** `gentity_t::reset_fn_ids_after_zero()` (single source of truth in gentity.rs) sets all 7 fields to `None`; called at every whole-entity zeroing site: `G_FreeEntity`, `G_InitGentity`, `g_init_game.rs:191` (InitGame array memset), and both `GameWorld::zeroed*` constructors. The misleading STATE-D9 "all-zero bytes are a valid gentity_t" contract comment was corrected.
**Follow-up (recommended):** type-level fix — NonZero-backed handler ids (guaranteed 0 == None niche) so no future memset can resurrect this class. Also audit other structs for `Option<enum>` fields living in memset'd memory.

## Verification (live, instrumented rig)
Post-fix round: trigger touch → USE-BINARY → MOVERGO with textbook `delta=(0,0,500)` on ents 114/115; `pos2` stable all session (watchdog); elevator visible, rides correctly; client MISSVEC = spawn teleport only. User confirms: "it was that."

## Cleanup checklist for next session
- Remove all `PROBE(seam-test)` probes from jka-rust working tree (g_active.rs, g_mover.rs, g_main.rs, g_trigger.rs, g_utils.rs, trap.rs, jampgame/lib.rs) — KEEP the two real fixes (GlobalUse + reset_fn_ids_after_zero) and consider keeping the panic hook (silent unwind across C-unwind exports is a real hazard).
- Commit the two fixes separately with reference to this doc.
- Remove OpenJK worktree: `git -C ~/Developer/Milo/lmd-rewrite/OpenJK worktree remove --force /private/tmp/claude-502/-Users-milohehmsoth-Developer-Milo-jedi-academy-rust/3aab0b49-6372-4274-8ff7-033bf29df3ea/scratchpad/openjk-seam` (contains the SEAM/RSEAM/probe patches — harvest anything wanted first).
- `~/Developer/jka/seam-test/` rig can stay (useful for future live testing); round logs archived there.
- The original taystjk symptoms (constant 17.88 standing miss from Jul 6/7 morning) should be re-tested against the fixed module — likely the same root cause via movers near spawn; if a standing-still miss persists, that's a separate residual.

---

# SESSION 3 (2026-07-07 night) — automated A/B referee rig built and operational

Task list (in Claude session): #4 rig (in progress, nearly done) → #5 probe cleanup + commit fixes → #6 stub-triage → #7 NonZero handler ids → #8 panic policy → #9 syscall audit → #10 taystjk re-test → #11 entity regression sweep. Process: plan before executing each.

## The rig (replaces human playtesting)
- **Engine referee layer**: `openjk-seam` worktree, `codemp/server/sv_referee.cpp/.h` + ~8 one-line hooks. Cvars: sv_stepper (lockstep frame gate, forced fixed msec), sv_recordInputs / sv_replayInputs (usercmd record/replay incl. bots; replay spawns fake HUMAN clients on NA_BOT netchans, isBot=false), sv_refSeed (GAME_INIT seed override; replay defaults to header seed), sv_refTrace (ABI boundary ring buffer). Commands: ref_step [n] → `REF STEP n= t= ck=` sentinel/frame (FNV-1a over all entityState+entityShared+playerState); ref_dump (full hex state); ref_trace (per-frame vmMain IN / syscall OUT records). Forces legacy vmMain ABI when active. Run with +set com_timestamps 0. Cvar-gated → zero cost idle; permanent keeper.
- **Orchestrator**: `~/Developer/jka/seam-test/referee/` — referee.py (lockstep step/compare, divergence report with field names from embedded ES/PS offset tables, trace-fork context, REPL with pids for lldb attach), run-ab.sh, record_corpus.py, README. Use --batch 1 and explicit --seed matching corpus header.
- **Corpus**: referee/corpus-ffa1.rec — 4 bots × 2400 cmds, seed 42, mp/ffa1.

## First A/B verdict (Rust vs OpenJK-game): divergence at frame 1 — but CONFOUNDED
OpenJK's game ≠ Raven oracle. Mixed findings: OpenJK drift (ClientConnect does 1283 vs 569 boundary records, spawn eFlags/pitch/groundEntityNum init, 256 vs 1024 bufsize) vs likely-real port findings: **ents 117-128 loopSound/eventParm = 0 on Rust vs 7-10 (ambient speaker sound/configstring indices missing)**; **-0.0 vs +0.0 in r.mins[2] on 44 item ents**; 1-ULP velocity drift on client 0 at frame 1.
Decision (user): **ignore OpenJK as reference; oracle only.** Oracle source = oracle/oracle/ = mheh/jediacademy @4bebb8e (nested inside the oracle wrapper crate submodule, mheh/jedi-academy-rust).

## In flight at compaction
Background agent building the **oracle game dylib** (oracle/oracle/codemp/game/*.c → arm64 jampgamearm64.dylib, exports vmMain/dllEntry, -ffp-contract=off, harness in scratchpad/oracle-game-build/, reuse oracle/build.rs config if possible, do NOT modify the vendored oracle) and re-running the 2400-frame A/B as Rust-vs-oracle. Its report = the port's true baseline diff. Referee polish backlog: mask trace arg slots beyond each trap's real arg count (C stack garbage vs Rust zero-fill = noise).

## Session logistics
- Session runs from jedi-academy-rust cwd (can't re-root); all jka-rust work via absolute paths. Next session: open in ~/Developer/Milo/jka-rust directly.
- jka-rust tree state: HEAD ddc598be + uncommitted: the two REAL fixes (GlobalUse dispatch; reset_fn_ids_after_zero at 5 zeroing sites) + ~27 PROBE(seam-test) probes + panic hook (jampgame/lib.rs). Task #5 separates and commits.

---

## SESSION 3 (cont.) — Rust-vs-ORACLE baseline landed (task #4 COMPLETE)

Reference side is now the vendored Raven oracle game (`oracle/oracle/codemp/game/*.c`), per user
directive "Ignore OpenJK in this regard for now. Do only oracle."

### Oracle dylib build (reusable harness)
- Repo already ships a full-module harness: `tools/referee-oracle/build.sh` (89 game TUs → arm64
  dylib exporting `_vmMain`/`_dllEntry`). Agent copied it to scratchpad
  (`scratchpad/oracle-game-build/build.sh`), retargeted ORACLE path; vendored tree untouched —
  all patches applied to a throwaway copy under `build/src/`.
- Compiler: Homebrew **g++-16** (Apple clang lacks `-fpermissive`, needed for Raven's 32-bit-era
  FOFS ptr→int casts). Flags: `-x c++ -std=gnu++98 -fpermissive -O2 -fno-fast-math
  -ffp-contract=off -fsigned-char`; defines `-DQAGAME -D_JK2MP -D__linux__ -DNDEBUG`.
- Two REQUIRED patches to the copy:
  1. `g_main.c:515` vmMain args `int` → `intptr_t` (engine passes pointers through arg slots;
     `GAME_NAV_CLEARPATHTOPOINT` reentrant callback truncated a stack ptr → SIGSEGV in
     `NAV_ClearPathToPoint → trap_InPVS`). Same widening OpenJK applied; zero logic change.
  2. `q_shared.h:1404` linux `SnapVector` macro truncates (`(int)`); retail MSVC fld/fistp, the
     Rust port (`round_ties_even`, bg_misc.rs:1506), and the engine trap all round-to-nearest-even.
     Replaced with `rintf`. Pre-fix, all 4 players diverged at frame 1 in apos.trBase pitch
     (−2.0 vs −1.0 = round vs truncate of ≈−1.5).
- `run-ab.sh` C_DYLIB now → oracle dylib; OpenJK game preserved as
  `side-c/base/jampgamearm64-openjk.dylib`.

### VERDICT: divergence at frame 1 (t=425), 0 frames pass — both families are rust-port issues
1. **G_SoundIndex/G_ModelIndex/G_EffectIndex are placeholder stubs returning 0**
   (`crates/mp/game/src/g_utils.rs:252`, PORT-NOTE ctx-free-boundary-callee-mismatch).
   Symptoms: ents 117–128 (target_speakers) `loopSound`/`eventParm` = 0 vs 7/8/9/10;
   trace fork right after `GAME_CLIENT_BEGIN(0)`: C emits ~87 extra configstring syscalls
   (`G_FindConfigstringIndex` registration loop) the stubs never perform (569 vs 1326 records).
   **Same stub class as GlobalUse — feeds task #6.**
2. **−0.0 vs +0.0 in `r.mins[2]`** on 57 item ents (byte 563 sign bit): `g_items.rs:1764/:3796`
   write `[-8.0, -8.0, -0.0]`; C's `VectorSet(..., -8, -8, -0)` is integer −0 → **+0.0f**.
   Bit-level; fix rust to `0.0`.
3. Protocol-only, not a bug: rust routes SnapVector through engine trap `G_SNAPVECTOR` (record 16),
   C inlines the macro — rounding identical post-patch. Trace-differ noise: C leaves stack garbage
   in unused arg slots (e.g. 2³²+1024 vs 1024) — masking polish still pending.
- Bonus: OpenJK-run's extra diffs (eFlags+3, fireflag, groundEntityNum) are GONE vs oracle —
  the oracle module is strictly closer to the port than OpenJK's game.
- Logs: `scratchpad/ab-oracle-run{,2,3}.log`, engine logs `ab-oracle-run3-{rust,c}-engine.log`,
  crash bt `lldb-crash-out.log`; live logs `~/Developer/jka/seam-test/referee/logs/`.

### Next (plan-before-execute at each step)
Proposed order: fix the two frame-1 findings (index stubs + −0.0) → rerun A/B to get a real
PASS depth → then task #5 (strip probes, commit) → #6 stub sweep with rig as verifier.

---

## SESSION 3 (cont. 2) — Frame-1 findings FIXED; rig verdict: PASS 2400/2400

### Fixes (uncommitted, in working tree alongside the probes)
1. Index functions un-stubbed (`crates/mp/game/src/g_utils.rs`): the faithful
   `G_FindConfigstringIndex(ctx, ...)` already existed at :166 — only the ctx-free wrappers were
   stubs. `G_ModelIndex` (:225, CS_MODELS/MAX_MODELS), `G_SoundIndex` (:249, CS_SOUNDS/MAX_SOUNDS),
   `G_EffectIndex` (:269, CS_EFFECTS/MAX_FX) now dispatch for real.
   Ctx-free trap pattern: `g_strap.rs` STRAP_ENGINE OnceLock (armed in g_init_game);
   `strap_engine()` made pub(crate) (:51); wrappers build
   `GameContext { world: null_mut(), engine: strap_engine() }` — same null-world precedent as
   `GameBgTraps::new`. Safe because G_FindConfigstringIndex touches only ctx.engine.
   Added `MAX_MODELS=512`, `MAX_SOUNDS=256`, `MAX_FX=64` to
   `crates/mp/bg/src/public/configstring.rs` (from q_shared.h:2020-2023).
2. Negative zero: `g_items.rs:1764/:3796` `-0.0` → `0.0` (C integer `-0` promotes to +0.0f).
   Crate-wide grep: no other bad `-0.0` literals.

### Verdict
**PASS 2400/2400 frames, zero divergence, zero trace forks** (Rust vs Raven oracle dylib,
corpus-ffa1.rec, seed 42, batch 1, 67s). The previously-forking GAME_CLIENT_BEGIN window
(ents 117-128, ~87 configstring syscalls) is now byte-identical.

Residual (non-lockstep) observation: shutdown G_Alloc pool tallies differ slightly
(rust 919 blocks / 12,304,770 B vs c 932 / 12,304,977 B) — allocation-count delta worth a
future look, not part of frame comparison.

### Status: task #4 done and validated. Next: task #5 (strip ~27 PROBE(seam-test) probes,
decide panic-hook fate, commit fixes separately, NO co-author trailers), then #6 stub sweep
using the rig as verifier.

---

## SESSION 4 (2026-07-07 → 07-08) — fix batches landed, engine planned, const sweep staged

### Commits on `skeleton` (chronological)
Pushed to origin (origin/skeleton = de0bb6bd):
- 95e14541 GlobalUse dispatch + handler-id reset  · 79be9e66 index fns  · 1f1d9c97 −0.0 item mins
- 4f65a23e batch 1: G_AddEvent eventTime + G_PlayEffect/G_PlayEffectID (STRAP_WORLD seam)
- f1fa7cb1 batch 2: 5 dropped vec3 out-params (3 audited + NPC_GetMoveDirection{,AltRoute} found
  by scripted sweep of 481 by-value vec3 params)
- 3dbede1b batch 3: vehicle cluster (vtable already existed as veh_dispatch.rs — mechanical wiring;
  Gonk no-op is faithful, Raven commented it out)
- 09afce35 batch 4: NPC_Pain anim timing, BG_ValidateSkinForTeam file-gate, PM_SetSaberMove stance,
  TryHeal, dispensers
- 06afe785 FnId<E> NonZero-backed handler ids (repr(transparent) Option<NonZeroU8>, None==0
  std-guaranteed; reset_fn_ids_after_zero DELETED; zeroed-gentity unit test)
- de0bb6bd panic policy: catch_unwind at vmMain → Com_Error path; panic hook at dllEntry;
  panic="unwind" pinned in workspace profiles (pre-change: panics unwound into C = UB)
Also pushed (72c25697, 8e0cf1aa — landed after this session, no longer local-only):
- 72c25697 TryUse real consts (GT_SIEGE 4→7, SETANIM_TORSO 2→1, CLASS_VEHICLE 0→real,
  STAT_MAX_HEALTH 1→8, HI_JETPACK 2→7, PMF_FOLLOW 0→4096, content masks 0→real, .value→.integer)
  + Use_Target_Delay null-activator UB fix (ent_id→ent_id_opt)
- 8e0cf1aa g_utils.rs fresh-eyes const audit: zero wrong values remain; 2 shadowing consts with
  LYING "no canonical" comments removed (PMF_TIME_KNOCKBACK, SVF_BROADCAST)

### Verification state
- Rig regression: PASS corpus-ffa1 2400/2400 AND corpus-ffa1-combat 4800/4800 vs oracle dylib
  (also re-run after 72c25697: PASS).
- User's live taystjk retest (task #10): "Everything runs fine" — original bug CLOSED.
- Static review of newly-live code (4/5 clusters clean; all confirmed bugs were
  TryUse consts, fixed above) — findings folded in above; doc removed, see git
  history.

### Rig now scratchpad-independent
~/Developer/jka/seam-test/referee/artifacts/ holds openjkded.arm64 (patched engine),
oracle-jampgamearm64.dylib, oracle-build.sh, openjk-referee-patches.diff (1049-line full engine
patch series incl. untracked sv_referee.cpp/.h). run-ab.sh repointed there.

### Plans drafted (all UNCOMMITTED in docs/plans/)
- 2026-07-07-rust-referee.md — PARKED; status block holds the final architecture (referee lib+bin
  crate on a game-host interface crate; deep-gameplay strategy).
- 2026-07-08-mp-engine-build-out.md — engine port plan from full walker sweep (2756 fns/93k LOC,
  DAG: 20 waves, 1389 leaves; ghoul2⇄renderer welded → one tier; botlib/icarus near-isolates →
  tier 4; C++ subsystems need object-model port, invisible to call graph). Walker gained
  mp-engine-ded profile + enginesweep.py (uncommitted in tools/closure-prototype/).
- task #18 process (unmarked-placeholder-const sweep); doc removed, see git history.

### Standing rules added this session (memories + global CLAUDE.md)
- Subagents ALWAYS on explicit opus/sonnet/haiku (never inherit session model) — global.
- Every agent prompt: also check suspicious local consts/enum literals vs oracle + canonical
  crates (memory: agent-prompts-check-placeholder-consts).
- FnId pattern replaces the Option<enum> memset hazard (memory updated).

### Closure note (2026-07-08)
Task #18 (unmarked-placeholder-const sweep) completed — see
`docs/audits/const-sweep-2026-07-08.md` and `docs/audits/gatesweep-2026-07-08.md`.
This investigation is CLOSED.
