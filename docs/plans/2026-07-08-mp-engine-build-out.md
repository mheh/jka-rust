# MP engine build-out plan — pure-Rust dedicated JK2 server

Status: DRAFT 2026-07-08. Nothing here is built yet. This plan sequences the
port of the **codemp dedicated-server engine** (the `openjkded` half of the
seam) into Rust, so the already-ported game module (`crates/mp/game`, verified
byte-faithful to the Raven oracle — see the referee sessions in
`docs/handoffs/2026-07-07-prediction-miss-investigation.md`) can eventually run
on a Rust host instead of a patched C `openjkded`.

It is grounded in a mechanical walk of the actual dedicated-server sources
(`tools/closure-prototype/enginesweep.py`, profile `mp-engine-ded`) whose output
lives in `tools/closure-prototype/out/engine/`. Read that stats file
(`engine-fn-stats.md`) alongside this plan — every number below comes from it.

This plan is deliberately consistent with, and downstream of,
`docs/plans/2026-07-07-rust-referee.md`: **the game-host interface crate is built
first; the referee and the real engine crates both consume it; one engine lives
in the repo, not two.**

---

## 0. What the walker found (the data this plan rests on)

One unity libclang parse of the WinDed.vcproj Release compile set (qcommon +
server + ghoul2 + botlib + icarus + RMG + the 9 model-loading renderer sources
WinDed links, `-DDEDICATED -DBOTLIB`, `-fno-operator-names -fdeclspec`):

- **2,756 functions/methods** (764 are C++ methods), **92,595 LOC** of function
  extent, **121 function-scope statics**, **682 file-scope globals**
  (most mutable — the shared engine state a port must thread).
- Whole-engine call graph: 2,756 nodes, 3,934 resolved in-engine edges,
  **2,675 SCCs** (only 4 non-trivial, largest 78) → the engine is overwhelmingly
  a DAG. **20 topological waves; 1,389 functions are leaves (wave 0)** — half the
  engine calls nothing else in-engine and can be ported bottom-up in parallel.

Per-subsystem (fns / methods / body-LOC / files):

| subsystem | fns | methods | LOC | files | fn-statics | SCCs (largest) | waves |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| qcommon | 867 | 169 | 27,213 | 36 | 50 | 840 (28) | 12 |
| server | 168 | 0 | 7,367 | 9 | 12 | 166 (3) | 9 |
| ghoul2 | 224 | 11 | 9,230 | 5 | 40 | 224 (1) | 7 |
| botlib | 697 | 0 | 24,065 | 28 | 8 | 697 (1) | 15 |
| icarus | 540 | 463 | 10,983 | 11 | 1 | 540 (1) | 6 |
| RMG | 113 | 108 | 3,856 | 12 | 1 | 113 (1) | 1 |
| renderer (G2/model subset) | 147 | 13 | 9,881 | 8 | 9 | 143 (3) | 8 |

**Cross-subsystem call matrix** (caller→callee resolved edges — the tier oracle):

| caller \ callee | qcommon | server | ghoul2 | botlib | icarus | RMG | rend |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| qcommon | *1179* | 12 | 1 | · | · | · | 3 |
| server | 305 | *184* | 23 | 2 | 2 | · | 4 |
| ghoul2 | 25 | 2 | *295* | · | · | · | 66 |
| botlib | 75 | · | · | *1456* | · | · | · |
| icarus | 24 | 6 | · | · | *53* | · | · |
| RMG | 30 | 1 | · | · | · | *0* | · |
| renderer | 86 | · | 11 | · | · | · | *125* |

Three structural facts fall straight out of the matrix and set the tier
boundaries:

1. **qcommon is a true leaf.** It makes 1,179 intra-subsystem calls and only ~16
   calls into everything else combined (12 into server — these are the handful of
   `SV_*` callbacks qcommon's cm/event code invokes; 3 into renderer). Port
   qcommon and you owe almost nothing else. It is the floor of every tier.
2. **botlib and icarus are near-isolates over qcommon.** botlib: 1,456 internal
   edges, 75 into qcommon, **zero** into anything else. icarus: 53 internal, 24
   into qcommon, 6 into server. Both are cleanly deferrable to their own tier —
   nothing in Tier 1–3 calls into them.
3. **ghoul2 ⇄ renderer are welded together** (ghoul2→renderer 66, renderer→ghoul2
   11). Server-side Ghoul2 is not separable from the model/mesh/shader loader
   that WinDed drags in from `renderer/` — they must be one tier.

Dispatch tables (indirect-call seams the port must resolve explicitly), found by
counting `field = Fn` assignments per function:

- **botlib import/export**: `Init_AI_Export` (75), `Init_EA_Export` (26),
  `Init_AAS_Export` (22), `GetBotLibAPI` (15) in `be_interface.cpp`;
  `SV_BotInitBotLib` (17) in `sv_bot.cpp` populates `botlib_import_t`.
- **renderer API**: `GetRefAPI` in `tr_init.cpp` builds `refexport_t` (only 1
  assign survives under `DEDICATED` — most `re.X = RE_X` compile out, itself a
  useful signal of how thin the dedicated renderer surface is).
- **ICARUS game interface**: `Interface_Init` (40) in `Q3_Interface.cpp`.
- **VM dispatch is NOT a table**: it is the `SV_GameSystemCalls` switch (1,200
  LOC, the single largest function in the engine) plus the `vmMain` trampoline.
  This is the seam the interface crate models.

---

## 0.5 Existing seeds to build on (what is already in the tree)

The port is not starting from zero. `crates/mp/engine/` already has **9
subcrates** with the dedicated-server spine partly wired; this plan fills in the
`todo!` bodies leaf-first. Live vs stub, from the current tree:

**Live and test-exercised (the working scaffold):**
- **Module loader** — `crates/native/platform/src/module_loader/` `sys_load_dll`
  loads the real game dylib, resolves `dllEntry`+`vmMain`, hands over the syscall
  trampoline. `RawVmMain`/`RawDllEntry` are 12-word `extern "C-unwind"`. Works.
- **Inbound syscall trampoline** — `crates/mp/engine/qcommon/src/vm/`: a
  C-variadic half (`game_syscall_trampoline.c`, faithful `VM_DllSyscall` copy,
  unpacks va_list → `intptr_t args[16]`) + a Rust half (`trampoline.rs`) that
  dispatches through the armed `GAME_SLOT`. **`arm_game_slot(ctx, syscall)`** and
  `ModuleRegistry::{load_module, vm_call}` are live (flagged PROVISIONAL);
  `VM_Restart` is a `todo!`. The syscall-number enum (`MpGameImport`) lives in
  `crates/mp/abi`, matched by the server dispatcher.
- **`Engine` aggregate + partial lifecycle** —
  `crates/mp/engine/core/src/engine.rs`: `Engine { common, sv, cl, cm, snd }`
  (`cl`/`snd` = `None` on dedicated). `com_init` boots a stubbed 42-step contract
  (only steps 3/5/7/12 wired as no-ops); `sys_milliseconds`/`sys_error` work.
- **The mock engine = the behavioral syscall spec** —
  `crates/jampgame/tests/common/mod.rs` (~1066 lines; **READ-ONLY, another agent
  owns crates/jampgame**). A `MockEngine` (thread-local, `BTreeMap`-backed for
  determinism) that answers the full game→engine syscall surface: cvar family,
  `G_ARGC/ARGV`, FS opens, configstrings, `G_GET_ENTITY_TOKEN` (streams a
  worldspawn+spawns map), **`G_LOCATE_GAME_DATA`** (captures the entity/client
  array bases), `G_GET_USERCMD`, and **`G_TRACE/G_TRACECAPSULE/G_G2TRACE`**
  (writes an empty-space `trace_t`: `fraction=1.0`, `entityNum=ENTITYNUM_NONE`).
  **This is the exact contract the Tier-1 real implementations must satisfy** —
  each real `SV_*`/`CM_*`/`FS_*` replaces one mock arm.
- **The in-repo Rust A/B referee** — `crates/jampgame/tests/referee.rs` already
  runs the Raven oracle dylib (`tools/referee-oracle/build/`) and the Rust game
  cdylib under the one mock engine and byte-diffs every `playerState_t`/
  `entityState_t` per frame + a syscall-stream FNV-1a digest. This is live and is
  the harness the engine swap-in tests extend (§3c).
- **Oracle parity harnesses** — `tools/jampgame-oracle` (golden IEEE-754 bit-
  pattern dumps from unmodified Raven `.c`; `crates/mp/game/tests/*_parity.rs`
  reproduce them), `tools/gp2-oracle` (GP2 parser), the `oracle/` submodule's
  in-process C-FFI wrappers (`oracle/build.rs` → `libja_oracle.a`), and
  `tools/referee-oracle/build.sh` (builds Raven's jampgame as a dylib).

**Class-A stubs to fill (the Tier-1 work list, already stubbed with the right
signatures):**
- `crates/mp/engine/server/src/server_host.rs` — **`sv_game_system_calls`** (the
  `SV_GameSystemCalls` dual; only `G_PRINT` handled, rest `todo!`) and
  **`game_system_calls_shim`** (the injected `SlotSyscall`; null-ctx path stubbed).
  Also `Server { sv: server_t, svs: serverStatic_t, world_sectors, bot, ... }`.
  (There is **no standalone `server_host` crate** — it is this module.)
- `crates/mp/engine/core/src/lifecycle.rs` — `com_frame_body`, `com_shutdown`,
  `com_error_recover` are `todo!`; `sv_init_game_progs` injects `ctx = null_mut()`
  (PROVISIONAL — the ctx must become the real `ServerGame`).
- `crates/mp/engine/qcommon/src/common/boot_stubs.rs` — `cvar_init`/`cbuf_init`/
  `cmd_init`/`fs_init_filesystem` are no-op boot-success stubs.

**Type-level presence (struct layouts largely ported by Wave 5, logic empty):**
qcommon (107 files — cm/collision, files/pk3, msg/net/huffman, gp2, qfiles/BSP,
vm — mostly struct-only), botlib (52), icarus (20), ghoul2 (13), rmg (5, enums
only — C++ classes deliberately out of scope), client (49). **Renderer is NOT
under engine/** — it lives in `crates/mp/renderer`; the Tier-2 G2/model-loading
slice draws from there.

So: the **data model exists, the dispatch spine exists, the bodies do not.** This
plan is a body-filling plan, ordered by the walker's waves, gated by the referee
that is already built.

---

## 1. Scope and tiers

The deliverable is a headless, deterministic, referee-grade Rust host that loads
the existing game dylib and answers its syscalls byte-identically to the C
`openjkded`. Networking, bots, scripting, and terrain are strictly later tiers.

### Tier 0 (prerequisite, from the referee plan) — the game-host interface crate
Not engine code; the contract. A `crates/mp/host-interface` (name TBD) crate of
Rust traits transcribing the C seam: the **syscall surface** (the `SV_*`/`CM_*`/
`FS_*` functions the game reaches through `trap_*` = the `MpGameImport` numbers in
`crates/mp/abi`), the **vmcall driver** (`GAME_INIT`/`GAME_RUN_FRAME`/
`GAME_CLIENT_THINK`/... via the existing `RawVmMain` 12-word trampoline), and the
**shared-memory contract** (`G_LOCATE_GAME_DATA`, the `gentity_t`/`gclient_t`/
`playerState_t` stride agreement already asserted in the port). Both the referee
harness and every real engine crate implement/consume this trait — this is the
"one marshaling dispatcher" (referee-plan task #9) realized once. **Build this
before Tier 1.** Two working seeds converge here: the **mock engine**
(`crates/jampgame/tests/common/mod.rs`) is the behavioral spec of exactly this
trait, and the **dispatch spine** already exists (`arm_game_slot` +
`game_system_calls_shim` + `sv_game_system_calls` in `server_host.rs`). Tier 0 is
extracting the mock's arms into a real trait and making `sv_game_system_calls`
dispatch it with the real `ServerGame` ctx (today it injects `null_mut()` and
`todo!`s every arm but `G_PRINT`).

### Tier 1 — "referee-grade headless host" (~25k LOC target; qcommon + server core)
A server that loads a map, spawns the game, runs the client-think/frame loop, and
answers traces/pointcontents/configstrings — enough to reproduce the referee
rig's `ab_idle_baseline` and re-verify game fix batches 1–4 against a **pure-Rust**
host instead of the patched C engine. **No net, bots stubbed, Ghoul2 stubbed.**

Input dossiers (unresolved design forks for this tier):
`docs/dossiers/B1-cvar-cmd.md`, `docs/dossiers/B2-filesystem.md`,
`docs/dossiers/B3-collision.md`, `docs/dossiers/B5-server.md`.

In-scope subsystems, leaf-first within qcommon's 12 waves:
- **Filesystem/pk3** (`files.cpp`, `unzip`): `FS_*`, pk3/zip mounting, the pure-
  server pak list. The largest single function is `FS_FOpenFileRead` (324 LOC).
- **Collision** (`cm_*.cpp`): map/BSP load, brush/patch/terrain trace,
  pointcontents, leaf/area queries. `CM_Trace` (253), `CM_LoadMap_Actual`
  (carries the `last_checksum` static — a determinism cell). This is the heart of
  the syscall surface the game hammers (`trap_Trace`, `trap_PointContents`,
  `trap_InPVS`, `trap_AreaEntities`).
- **sv_world entity linking** (`sv_world.cpp`): the areagrid, `SV_LinkEntity`/
  `SV_UnlinkEntity`/`SV_ClipMoveToEntities`/`SV_AreaEntities`. This is where the
  Session-2 "invisible elevator" bug lived (link → absmin → box-query seam), so it
  is a known-high-risk parity target with an existing reproducer.
- **Syscall implementations** (`sv_game.cpp`): `SV_GameSystemCalls` (the 1,200-LOC
  switch), `SV_LocateGameData`, `SV_GameSendServerCommand`, configstring get/set.
- **Frame + client-think loop** (`sv_main.cpp`/`sv_init.cpp`/`sv_client.cpp` frame
  parts): `SV_SpawnServer` (320), `SV_Frame`, `SV_ClientThink`, the vmMain driver.
- **cvar/cmd/common/msg-lite** (`cvar.cpp`, `cmd.cpp`, `common.cpp`, `q_math`,
  `md4`/`crc`): the util floor everything sits on (wave-0 leaves).

Deferred inside Tier 1: the VM bytecode interpreter (`vm.cpp` `VM_Compile` 698 /
`VM_CallInterpreted` 610) — the game loads as a **native dylib**, not QVM, so the
interpreter is not on the critical path; only the `VM_Call`/`vmMain` native
trampoline is needed. This alone removes the two largest qcommon functions from
Tier 1.

### Tier 2 — server-side Ghoul2 + its renderer loader (+~15k LOC)
Un-stub Ghoul2 so server-side bone/bolt/collision queries are real. Because the
matrix welds ghoul2 to the renderer G2/model loader, this tier is
`ghoul2/G2_*.cpp` (224 fns, incl. `G2_RagDoll*` 485/417/286, `G2_TransformBone`)
**plus** the WinDed renderer subset that actually links under `DEDICATED`
(`tr_model`/`tr_mesh`/`tr_ghoul2`/`tr_shader` model + `.md3`/`.glm`/`.mdx` parsing;
`ParseStage` 642, `FinishShader` 335 are shader-text parsing, not GL). Ghoul2
carries **40 function-scope statics** (the most of any subsystem — the RagDoll
solver is riddled with them), so it is the highest-effort-per-LOC tier.

### Tier 3 — real networking (+ msg/huffman/netchan/win_net.cpp+unix_net.c + sv_snapshot/sv_client)
The wire. `msg.cpp` (`MSG_ReadDeltaPlayerstate` 238, delta entity/playerstate
encode/decode), `huffman.cpp` (the `msgHuff` 102KB static table), `net_chan.cpp`,
`win_net.cpp`/`unix_net.c`, and the server snapshot/client half (`sv_snapshot.cpp`
`SV_BuildClientSnapshot`, `sv_client.cpp` `SV_DirectConnect` 348, `SV_ExecuteClientMessage`).
Unlocks a real network client (`taystjk`) connecting to a pure-Rust server. This
is where the SnapVector/`MSG_WriteDeltaPlayerstate` byte-faithfulness lessons from
the handoff become load-bearing again.

Input dossier (unresolved design forks for this tier): `docs/dossiers/B4-network.md`
(+ `docs/dossiers/B5-server.md`'s snapshot/client half).

### Tier 4 — botlib / icarus / RMG (each independently deferrable)
Per the matrix, nothing below calls up into these, so each ships on its own
schedule:
- **botlib** (697 fns, 24k LOC, 15 waves): AAS route/reachability/movement.
  Only needed when server-side bots are wanted. Depends solely on qcommon (75
  edges) + the `botlib_import_t` table `SV_BotInitBotLib` fills.
- **icarus** (540 fns, 463 methods, C++ class tree): ROFF/script sequencer. JK2
  MP uses it lightly; deferrable until scripted MP content matters.
- **RMG** (113 fns, 108 methods, virtual-dispatch class tree): random mission
  generation. OpenJK dropped it entirely (NOTES v10); lowest priority, may be
  cut.

---

## 2. Port order (leaf-first, within and across subsystems)

The walker gives a total order for free: **wave number ascending, ties broken by
subsystem-leaf-ness from the matrix.** Rules:

1. **Across subsystems:** qcommon (leaf) → server (integrator, depends only on
   qcommon) → {ghoul2+renderer} → {botlib, icarus, RMG} in any order. This is
   exactly the tier order; the matrix proves no back-edges violate it (server→
   ghoul2 is 23 edges but all reach *ghoul2 API* the Tier-2 crate provides;
   until then they are stubbed, which is precisely the Tier-1 "Ghoul2 stubbed"
   scope line).
2. **Within a subsystem:** port wave 0 first (leaves: pure math, string, single
   `cm_*` helpers), then wave 1, ... Wave 0 alone is 1,389 functions engine-wide
   and is embarrassingly parallel — the same "port-wave" agent fan-out that
   landed the type waves (NOTES v8–v11) applies unchanged, keyed on
   `engine-fn-manifest.json`'s per-function `wave` field.
3. **Non-trivial SCCs port as a unit.** There are only 4 in the whole engine
   (largest 78, inside qcommon's cm/patch code; small ones in server and
   renderer). Each SCC = a mutual-recursion cluster that must land in one commit;
   they are enumerated in the manifest.
4. **Statics and mutable globals are explicit port tasks, not incidental.** Each
   of the 121 function-scope statics is a hidden persistent cell (e.g.
   `CM_LoadMap_Actual::last_checksum`, the botlib `AAS_ContinueInitReachability`
   frame counters, Ghoul2's RagDoll `static` scratch). Each must be threaded
   through explicit state (a host struct field), never a Rust `static mut`, or
   determinism and reentrancy break. The 682 file-scope globals (`sv` 665KB,
   `tr` 316KB, `msgHuff` 102KB, `cvar_indexes`, the `fs_server*` pak arrays) are
   the shared-state struct fields of the host — they define the host's data
   model and should be laid out before their functions are ported.

---

## 3. Safety / testing methodology — "build it safe, test it to the oracle"

The port already has a proven parity discipline for the game module; this plan
reuses all four layers of it, aimed at engine functions. The ordering principle:
**pure functions get golden fixtures; stateful subsystems get captured-replay;
whole subsystems get engine-vs-engine lockstep before the next tier starts.**

### 3a. Per-function differential parity (oracle wrapper crate)
Reuse the `oracle/build.rs` + `oracle_c/*.c` parity-wrapper pattern (the
`tools/jampgame-oracle` golden-fixture harness) on engine functions that are
pure-ish: a thin C wrapper compiles the vendored oracle function, a Rust test
feeds both the same inputs and asserts byte-identical outputs.
- **Best fits (from the sweep's wave-0 leaves):** `cm_*` trace math
  (`CM_ClipBoxToBrush`, `CM_TraceThroughPatchCollide`), `msg.cpp` encode/decode
  (`MSG_WriteBits`/`MSG_ReadDeltaEntity` round-trips), `huffman.cpp`
  (`Huff_Compress`/`Huff_Decompress` against the fixed table), `q_math`,
  `md4`/`crc`. These are deterministic in→out and need no engine state.
- Generate the goldens from the same oracle build harness the game port uses
  (Homebrew g++-16, `-fno-fast-math -ffp-contract=off -fsigned-char`, per the
  handoff's oracle-dylib recipe) so float results match the reference exactly.

### 3b. Captured-fixture replay (sv_referee trace layer)
The patched `openjkded`'s `sv_referee.cpp` trace layer
(`scratchpad/openjk-seam`; documented in the handoff) already records **real
`trap_Trace`/syscall IN/OUT records** from live play (`sv_refTrace`, the per-frame
vmMain-IN / syscall-OUT ring). Replay those captured records as unit fixtures
against the Rust subsystem: feed the recorded inputs to the Rust `CM_Trace`/
`SV_LinkEntity`/configstring implementation and assert the recorded output. This
yields **thousands of real-world test vectors for free** — every trace the game
ever issued on ffa1 is a cm/sv_world regression case, with the exact float
inputs and outputs the C engine produced. This is the cheapest way to cover the
cm and sv_world seams (the two highest-risk Tier-1 targets) at scale.

### 3c. Subsystem swap-in acceptance (A/B lockstep)
The gate between tiers. Two A/B mechanisms exist, and the plan leans on the one
that is actually alive:
- **In-repo Rust referee (live, primary):** `crates/jampgame/tests/referee.rs`
  already runs the Raven oracle dylib and the Rust game cdylib **under one mock
  engine**, feeding identical `usercmd_t` streams and byte-diffing every
  `playerState_t`/`entityState_t` per frame plus an FNV-1a syscall-stream digest.
  For the *engine* port the seam flips: swap each newly-ported real engine
  subsystem in behind the interface trait (replacing that subsystem's mock arm),
  keep the rest as the trusted mock, and require the referee to stay byte-
  identical before the next subsystem starts. This needs no external process and
  runs under `cargo test`.
- **External engine-vs-engine lockstep (parked, richer):** the `sv_referee.cpp`
  layer on a patched `openjkded` (REF line protocol, `ref_step`/`ref_dump`/
  `ref_trace`, FNV-1a per-frame checksum) runs our Rust host and the C engine on
  the *same* dylib — the discipline that verified the game port (SESSION 3: PASS
  2400/2400). This is the ultimate acceptance test (real engine vs Rust engine),
  but its worktree (`scratchpad/openjk-seam`) is **currently gone** (session-
  ephemeral scratchpad) and must be reconstituted into a permanent home before it
  can gate anything — see risk #7. Until then, the in-repo Rust referee is the
  gate.

Concretely: as each subsystem lands it is swapped into the Rust host while the
rest stay trusted (mock, or later real C engine), and the A/B must be byte-
identical before the next subsystem begins.

### 3d. Determinism rules (carried over verbatim from the game port)
Non-negotiable, and several were learned the hard way in the handoff:
- **Fixed seeds** (`sv_refSeed`), forced fixed frame msec, `bot_enable 0`,
  `com_timestamps 0`. Reload-to-reset; no state serialization.
- **No wall-clock, no `HashMap` iteration order, no ASLR-dependent behavior** in
  ported code — these break byte-reproducibility across runs.
- **Float discipline:** `SnapVector` rounds ties-to-even (retail MSVC `fld/fistp`
  and `rintf`), **not** truncate — the linux macro's `(int)` truncation caused
  the frame-1 apos.trBase divergence. Compile the oracle reference with
  `-ffp-contract=off`; match `round_ties_even`. Watch integer `-0` promoting to
  `+0.0f` (the `r.mins[2]` item bug). 1-ULP and `-0.0` diffs are **findings, not
  noise**.
- **The `Option<enum>` memset niche hazard** (memory + handoff Bug 2): any engine
  struct with `Option<fnptr-enum>` fields living in `memset(0)` memory decodes as
  `Some(variant 0)`, not `None`. The engine has 682 globals many of which are
  zero-initialized; audit each `Option<enum>` field and reset-after-zero, or use
  NonZero-backed discriminants.

---

## 4. Milestones and unlocks

| # | milestone | subsystems done | unlock |
| --- | --- | --- | --- |
| M0 | interface crate + vmMain trampoline; game loads into a Rust process that immediately hands every syscall back to the C engine | Tier 0 | the seam is Rust-owned; A/B harness has a trait to swap against |
| M1 | FS + cvar/cmd + common util leaves port; Rust `FS_*` answers the game's file syscalls in A/B | qcommon waves 0–3 (files, util) | pk3/config loading is pure-Rust; oracle golden tests (3a) green for msg/huff/md4 |
| M2 | cm collision + sv_world linking + sv_game syscalls; **idle baseline reproduces PASS with the real Rust engine swapped in** for cm/sv_world/configstrings | Tier 1 complete (bots/G2 stubbed) | referee's engine-swap mode enabled; game fix batches 1–4 re-verified against a real Rust host (not just the mock); the invisible-elevator link→box-query seam owned in Rust |
| M3 | server-side Ghoul2 + renderer model/shader loader; G2 trace/bolt syscalls real | Tier 2 | `ab_combat_events`-class scenarios with real G2 collision pass A/B |
| M4 | msg/huffman/netchan/net_ip + sv_snapshot/sv_client | Tier 3 | **`taystjk` connects to a pure-Rust server over the network**; the original prediction-miss class re-tested end-to-end on Rust host+game |
| M5 | botlib / icarus / RMG as wanted | Tier 4 | server-side bots; scripted content; (RMG optional) |

Each milestone's gate is a green A/B lockstep run (3c) for the newly-swapped
subsystem, plus green oracle goldens (3a) and replay fixtures (3b) for it.

---

## 5. Risks and open questions (specific, from the walk)

1. **C++ subsystems in a C-centric pipeline (icarus 463 methods, RMG 108, ghoul2
   11).** The port pipeline (closure/portpacket/fnskel) is tuned for free C
   functions. icarus and RMG put nearly all logic in **class methods with virtual
   dispatch**, which the FUNCTION_DECL sweep originally missed entirely (RMG
   showed 5 functions until method collection was added) and whose **virtual
   calls do not appear as call-graph edges** — RMG's matrix row is `RMG→RMG = 0`
   because every internal call is a vtable dispatch. Consequence: (a) the wave/
   SCC structure for icarus/RMG **understates** their coupling; port order there
   must be derived from the class hierarchy, not the (empty) direct-call graph;
   (b) these subsystems likely want a hand-written Rust object model
   (trait objects / enums), not a mechanical transcription. This is the main
   reason they are Tier 4.
2. **botlib's AAS binary file formats.** botlib is 24k LOC and its determinism
   hinges on loading `.aas` area-awareness files (`be_aas_file.cpp`, the 78
   `LittleLong` byte-swap sites the sweep flagged). Parity requires bit-exact
   `.aas` parsing and the same route-cache behavior; the `AAS_ContinueInit*`
   frame-spread statics make it stateful across frames. High effort, low urgency
   (Tier 4, MP bots optional).
3. **Ghoul2 ⇄ renderer entanglement is structural, not incidental.** The 66
   ghoul2→renderer edges are real: `tr_ghoul2.cpp` (`G2_TransformBone` 511) lives
   in `renderer/` but is pure bone math the *server* needs for collision. WinDed
   compiles the real `tr_model`/`tr_mesh`/`tr_shader` under `DEDICATED`, so "the
   dedicated server has no renderer" is false — it has a headless model/shader
   loader. Tier 2 must port that slice, and the boundary between "model data the
   server needs" and "GL drawing the server doesn't" runs *through* individual
   files via `#ifndef DEDICATED`, not along file lines. Budget for reading those
   ifdefs carefully.
4. **The VM interpreter is skippable but the trampoline ABI is not.** Skipping
   `VM_Compile`/`VM_CallInterpreted` (native dylib load) removes the two biggest
   qcommon functions, but the `vmMain`/`SV_GameSystemCalls` marshaling must match
   the C ABI exactly, including the `intptr_t`-through-arg-slot widening the
   oracle build needed (handoff: `GAME_NAV_CLEARPATHTOPOINT` truncated a stack
   pointer). The interface crate must use `intptr_t`-width slots, not `int`.
5. **Parse coverage caveats (report them, don't trust silently).** The sweep ran
   with 1,258 benign diagnostics (MSVC `__asm` in `SnapVector` — unparseable on
   arm64, dropped; unity-TU header redefinitions across botlib `.cpp`; win32
   identifier shims). Function bodies and callees extracted cleanly, but **type
   layouts from this TU are not to be trusted** (that is `mp-engine`/Wave-5's
   job, done separately). The renderer numbers cover only the 9 WinDed-linked
   sources, not the full `renderer/` dir.
6. **Open: where does the interface crate's boundary sit** relative to the
   existing `crates/mp/engine/qcommon` trampoline seed — extend that crate or
   supersede it? (Resolve against the seeds map before M0.)
7. **The external openjkded rig is already gone.** The engine-vs-engine lockstep
   (3c, richer path) and the captured trace fixtures (3b) both depend on the
   patched `openjkded`/`sv_referee.cpp` in `scratchpad/openjk-seam` — which is
   **no longer on disk** (session-ephemeral scratchpad, per the seeds map). Its
   patch series must be reconstituted into a permanent home
   (`tools/openjk-referee/` or a documented patch set) before 3b/3c-external can
   gate anything. Near-term this is **not blocking**: the in-repo Rust referee
   (mock-vs-oracle, live) gates Tiers 1–2, and the captured `.reflog` fixtures in
   `tools/referee-oracle/logs/` already give directed input streams. But real-
   network acceptance (M4) and large-scale trace replay (3b at thousands-of-
   vectors scale) do need the C engine rig rebuilt — schedule that reconstitution
   before Tier 3.

---

## Appendix — reproducing the walk

```
cd tools/closure-prototype
.venv/bin/python enginesweep.py            # -> out/engine/{engine-fn-manifest.json, engine-fn-stats.md, subsys-*.json}
.venv/bin/python closure.py --list-modules # mp-engine-ded profile listed under [raven]
```

Profile `mp-engine-ded` (added to `closure.py`) is the WinDed.vcproj Release
dedicated-server compile set; `enginesweep.py` is the engine-scoped sibling of
`fnsweep.py` (collects C++ methods, per-subsystem breakdown, cross-subsystem
matrix, statics/globals census). Both are throwaway prototype tooling.
