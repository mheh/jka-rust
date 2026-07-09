# Referee plan: Rust A/B referee as a test target of `jampgame`

Status: PARTIALLY PARKED. The external engine-vs-engine rig (in-crate work below is
still parked 2026-07-07 pending resumption) has a permanent home again as of
2026-07-08: patched engine source is github.com/mheh/OpenJK branch `referee`
(commit f6d2875e "sv_referee: A/B module-parity referee layer" + 35e4184f
temporary debug probes), cloned at ~/Developer/Milo/OpenJK. Phase 0 completed
2026-07-08: engine rebuilt from that branch (`cmake -B build-referee`,
BuildMPDed only); both corpora re-verified — corpus-ffa1 PASS 4000 frames,
corpus-ffa1-combat PASS 5000 frames, zero divergence, oracle-built Raven dylib
vs Rust jampgame dylib. The driver (`~/Developer/jka/seam-test/referee/run-ab.sh`)
now points ENGINE at the from-source build; the old preserved binary is
retired. Scenario-matrix expansion beyond the two re-verified corpora remains
open. The in-crate rewrite described below (`crates/referee`) is still
unstarted.
Design discussion since writing superseded parts of this doc — capture before resuming:
- Rewrite around the EXISTING in-crate referee (`crates/jampgame/tests/referee.rs` + mock
  engine in `tests/common/`), not the external openjkded rig: referee becomes a lib+bin crate
  (`crates/referee`), CLI `referee <candidate.dylib> [frames] [--against] [--assets]`, thin CI
  gate remains in jampgame's tests.
- Build order: game-host interface crate FIRST (traits transcribing the C seam: syscall
  surface + vmcall driver + shared-memory contract; the one marshaling dispatcher = task #9
  realized), referee consumes it; real engine crates progressively replace stubs
  (FS → entity lump → collision). One engine in the repo, not two.
- Deep-gameplay strategy (external only): console-command vmcalls (npc spawn / give / god)
  to let the module generate its own gameplay; closed-loop scenarios steering off snapshots;
  time compression; fork-based deep-state branching; differential fuzzing (oracle = bug
  oracle, coverage-guided); semantic + line coverage as the honesty metric.

## Context and motivation

The lockstep A/B rig (see `docs/handoffs/2026-07-07-prediction-miss-investigation.md`,
SESSION 3) currently is:

- a patched `openjkded` (worktree `scratchpad/openjk-seam`, `codemp/server/sv_referee.cpp/.h`)
  providing: `sv_stepper`/`ref_step` lockstep stepping, `ref_dump` state hexdumps,
  `sv_refTrace`/`ref_trace` ABI-boundary tracing, `sv_refSeed` seed pinning, and
  `sv_recordInputs`/`sv_replayInputs` corpus record/replay;
- a Python orchestrator (`~/Developer/jka/seam-test/referee/referee.py`, ~1000 lines) that
  spawns two engine subprocesses (Rust module vs Raven-oracle module), steps them in lockstep,
  compares per-frame FNV-1a checksums, and on mismatch dumps + field-diffs both sides.

It works — it found and verified the GlobalUse stub, the Option-enum niche bug, the index-fn
stubs, and the −0.0 item mins — but two structural problems emerged during batch-1
(event/effect stub) verification:

1. **Corpus coverage is not directable.** A corpus recorded from live bot combat replayed
   3,100+ frames with *matching* checksums against a dylib with known-broken
   `G_AddEvent`/`G_PlayEffect` — replayed clients never fired (attack button lost somewhere in
   the record/replay client machinery). You cannot make recorded gameplay exercise a chosen
   code path.
2. **The Python differ duplicates layout knowledge.** Its field tables were hand-derived from
   `q_shared.h`; every layout change must be mirrored by hand, and the tables can silently rot.

Decision (user): retire recorded corpora in favor of **directly fed, synthesized inputs** at
the ABI boundary, and rewrite the orchestrator **in Rust as a secondary test portion of the
`jampgame` crate**.

## Core idea

- **Immutable starting point**: a `devmap <map>` load with pinned seed (`sv_refSeed`) and
  forced fixed frame msec *is* the deterministic snapshot. Both engines provably reach
  byte-identical state on every load (frame-0/1 checksums match across runs). No state
  serialization needed — reload to reset.
- **Inputs are synthesized, not recorded**: scenarios craft `usercmd_t` values in Rust each
  frame and inject the identical bytes into both engines just before each granted frame.
  Directed coverage becomes trivial: "hold `BUTTON_ATTACK` 50 frames", "walk onto the item at
  (x,y,z)", "press use at the door".
- **Engines stay in the loop as syscall oracles**: the game module's syscalls (traces against
  the BSP, configstrings, linking) must be answered by something; the two identical
  deterministic engines answer them. They answer identically for as long as the two modules
  behave identically — the first behavioral difference is exactly what the per-frame checksum
  and trace diff catch. (A no-engine tape/replay harness was considered and rejected for now:
  it requires a per-trap in/out-buffer descriptor table for ~100 syscalls and buffer-content
  capture. That idea is parked; the descriptor table overlaps with task #9 and may return.)

## Deliverable 1 — engine-side injection (small C patch, `sv_referee.cpp`)

The only unavoidable C work; extends the existing referee layer in the openjk-seam worktree:

- `ref_client <n> [userinfo]` — materialize a synthetic client in slot *n* without a netchan,
  reusing the existing replay-layer client-enter path (`type=2` handling): fake human client
  (`isBot=false`) on an NA_BOT-style local connection, `ClientConnect` + `ClientBegin` driven
  at the vmMain boundary.
- `ref_cmd <n> <serverTime> <p> <y> <r> <buttons> <fwd> <right> <up> <weap> [gsel] [fsel]` —
  build a `usercmd_t` from the args and feed it straight into `SV_ClientThink(n, &cmd)`.
  Angles as raw shorts (exact bytes, no float parsing ambiguity), moves as signed bytes.
- `ref_disconnect <n>` — drop the synthetic client (replay layer `type=3` path).
- While in this code: root-cause why `sv_replayInputs` cmds lost the attack button (first
  test: a single hand-crafted `ref_cmd` with `BUTTON_ATTACK` — if the game fires, the
  injection path is sound and the old bug was in record/replay framing, which is being
  retired anyway).

Everything else (stepper, dumps, traces, seed) already exists and is unchanged. The `REF`
line protocol is already stable and versioned by the `REF READY ... es= er= ps= cmd=` header.

Build: `cmake --build build --target openjkded.arm64 -j` in `scratchpad/openjk-seam`.
(Longer-term the worktree should move somewhere less ephemeral than the session scratchpad —
candidate: a `tools/openjk-referee/` submodule or documented patch series. Not in this step.)

## Deliverable 2 — Rust referee (`crates/jampgame/tests/`)

An integration-test target of the existing cdylib crate — no new workspace member; compiled
only under `cargo test`:

```
crates/jampgame/tests/
  referee.rs            # test entry: the #[test] fns + lockstep driver
  referee/
    engine.rs           # ServerProc: spawn openjkded, REF line protocol over stdin/stdout,
                        #   timeouts, engine-death detection (port of Python ServerProc)
    protocol.rs         # parse REF READY/STEP/DUMP BEGIN..END/TRACE; hex payload decode
    state.rs            # decode REF E (532B entityState + 112B entityShared) and REF P
                        #   (1552B playerState) payloads DIRECTLY into the workspace's own
                        #   #[repr(C)] types (from mp/qshared / mp/bg crates); startup
                        #   cross-check of size_of::<..>() against the REF READY es=/er=/ps=
                        #   values — the referee doubles as a live layout regression test
    diff.rs             # field-level dump diff using the real types (names come from code,
                        #   not hand tables); byte-diff fallback for padding regions; trace
                        #   diff with the two known noise masks:
                        #     - C stack garbage in unused syscall arg slots (mask args beyond
                        #       the per-trap real arg count — fixes the old polish item)
                        #     - rust-only OUT 48 G_SNAPVECTOR records (protocol-only)
    scenario.rs         # Scenario trait: fn cmds(&mut self, frame: u32) -> Vec<(slot, UserCmd)>
                        #   + combinators (hold buttons N frames, turn to yaw, walk toward a
                        #   point read from the previous frame's dump, sequence of phases)
    config.rs           # env-var config with current defaults:
                        #   JKA_REF_ENGINE   (default: scratchpad openjk-seam build)
                        #   JKA_REF_C_DYLIB  (default: scratchpad oracle-game-build build)
                        #   JKA_REF_BASEPATH (default: ~/Developer/jka/jka_server)
                        #   JKA_REF_WORKDIR  (side-a/side-b homepaths, default target/referee/)
```

Test behavior:

- Each `#[test]` stages the freshly built Rust dylib (cargo builds it as a dependency of the
  test target — always in sync) and the oracle dylib into two homepaths, spawns both engines,
  runs its scenario in lockstep, and asserts zero divergence. On divergence it prints the full
  field diff + first real trace fork (the Python report format, kept) and fails.
- Tests are `#[ignore]`d by default OR self-skip with a clear message when
  `JKA_REF_ENGINE`/assets are absent, so plain `cargo test` stays green on any machine.
  Invocation: `cargo test -p jampgame --test referee -- --ignored --nocapture` (exact gating
  mechanism decided at implementation time; requirement: zero friction locally, no CI breakage).
- Determinism requirements carried over verbatim: explicit seed, `--batch`-equivalent of 1
  (trace ring resets per frame), `bot_enable 0`, `com_timestamps 0`, wait for `REF READY`
  before the first `ref_step`, byte-exact policy (1-ULP and −0.0 diffs are findings).

Initial test suite:

| test | scenario | purpose |
|---|---|---|
| `ab_idle_baseline` | 4 synthetic clients, no input, 2400 frames on mp/ffa1 | replaces corpus-ffa1.rec PASS; rig sanity |
| `ab_combat_events` | clients aim at each other, fire all weapons, die, respawn, walk over pickups | **batch-1 verifier**: must FAIL against the current unfixed-event/effect dylib, PASS after the fix |
| `ab_movers_use` | walk into the ffa1 elevator, press use at doors | regression for the original prediction bug class |
| (later) `ab_vehicles`, `ab_siege` | vehicle map boarding/eject; siege class/heal | batch-3/-4 verifiers |

## Proof sequence (unchanged discipline)

1. Rust referee's `ab_idle_baseline` reproduces the Python rig's PASS.
2. `ab_combat_events` **diverges** against the current working tree's *stubbed* event/effect
   build — proving the scenario reaches the paths corpora never did. Divergence must be in
   event/effect fields (`event`, `eventParm`, `eventTime`-driven sweep effects, EV_PLAY_EFFECT
   temp entities).
3. Batch-1 fix (already written in the working tree: `G_AddEvent` eventTime via new
   `STRAP_WORLD` seam cell, `G_PlayEffect`/`G_PlayEffectID` temp-entity spawn; uncommitted)
   turns `ab_combat_events` green; `ab_idle_baseline` still green.
4. Commit batch-1 fix + the referee test suite (separate commits, no co-author trailers).
5. Python rig retired: `~/Developer/jka/seam-test/referee/` kept on disk as reference,
   README updated to point at `cargo test -p jampgame --test referee`.

## Execution plan (agents, per global model rule: explicit opus/sonnet/haiku)

1. **Agent A (opus)** — engine-side `ref_client`/`ref_cmd`/`ref_disconnect` + attack-button
   root-cause; rebuild openjkded; smoke-test by hand (one client, one attack cmd, confirm
   fire event in `ref_dump`).
2. **Agent B (opus, parallel after protocol pinned)** — Rust referee harness
   (engine/protocol/state/diff/config) + `ab_idle_baseline` reproducing the Python PASS.
3. **Agent C (opus, after A+B)** — `scenario.rs` + `ab_combat_events` + the proof sequence
   through commit.
4. Serial review + handoff-doc update between each.

## Open questions (decide before/at execution)

- Test gating: `#[ignore]` vs env-var self-skip vs cargo feature (`--features referee`).
- Where the openjk-seam worktree should permanently live (scratchpad is session-ephemeral;
  the built engine binary and oracle dylib likewise — they must move under
  `~/Developer/jka/seam-test/` or `tools/` before the scratchpad is garbage-collected).
- Whether `state.rs` should reuse the port's real structs directly (requires those crates as
  dev-dependencies of `jampgame` — they already are dependencies) vs a generated mirror.
  Preference: real structs, that's the point.
- Scenario aiming: closed-loop (read opponent origin from previous dump, compute angles in
  Rust) vs open-loop scripted angles. Closed-loop is more powerful and still deterministic
  (dumps are identical on both sides while in sync); start open-loop for simplicity.

## Current state at time of writing

- Batch-1 fix (`g_strap.rs` STRAP_WORLD, `g_init_game.rs` arming, `g_utils.rs`
  G_AddEvent/G_PlayEffect/G_PlayEffectID) landed as commit `4f65a23e`.
- Committed and rig-verified: GlobalUse fix, handler-id reset, index fns, −0.0
  (`95e14541`, `79be9e66`, `1f1d9c97`).
- Python rig + corpora at `~/Developer/jka/seam-test/referee/`; all engine processes down.
- Marker-triage findings ranked in `docs/audits/2026-07-07-marker-triage.md`; fix batches
  2–4 queued behind batch-1 verification.
