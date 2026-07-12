# MP engine build-out plan — pure-Rust dedicated JK2 server

Status: DRAFT 2026-07-08, REVISED 2026-07-09 (total scope: every function of
the WinDed link set, 1:1, **no `todo!()` stubs, no deferrals** — user
directive). Nothing here is built yet. This plan sequences the port of the
**codemp dedicated-server engine** (the `openjkded` half of the seam) into
Rust, so the already-ported game module (`crates/mp/game`, verified
byte-faithful to the Raven oracle — see the referee sessions in
`docs/handoffs/2026-07-07-prediction-miss-investigation.md`) can run on a Rust
host instead of a patched C `openjkded`.

It is grounded in a mechanical dependency walk of the actual WinDed compile
set (`tools/closure-prototype/engineorder.py`, profile `mp-engine-ded`) whose
output lives in `tools/closure-prototype/out/engine/engine-port-order.{json,tsv,md}`
— the **total port progression**: every function in bottom-up dependency
order, machine-verified so that porting in emitted order never references an
unported symbol. Every number below comes from it. (The earlier
`enginesweep.py` unity-TU walk is superseded for the graph — see §5.5 for the
three defects the rerun fixed — and retained only for its statics/globals
census.)

This plan is deliberately consistent with, and downstream of,
`docs/plans/2026-07-07-rust-referee.md`: **the game-host interface crate is built
first; the referee and the real engine crates both consume it; one engine lives
in the repo, not two.**

---

## 0. What the walker found (the data this plan rests on)

One libclang parse **per source file** (110 TUs, merged by USR) of the actual
`WinDed.vcproj` Release link set — qcommon + server (incl. `server/NPCNav/`) +
ghoul2 + botlib + icarus + RMG + the 9 model-loading renderer sources + the
`null/` dedicated client-stub layer; `-DDEDICATED -DBOTLIB`,
`-fno-operator-names -fdeclspec -fms-compatibility`:

- **2,481 functions/methods**, **87,728 LOC** of function extent, **119
  function-scope statics**, ~680 file-scope globals (most mutable — the shared
  engine state a port must thread; census in `engine-fn-stats.md`).
- Whole-engine dependency graph: **5,081 edges** (4,820 resolved calls + 261
  address-taken references — dispatch-table targets count as dependencies),
  **2,347 SCCs** (only 7 cyclic, largest 110) → the engine is overwhelmingly a
  DAG. **27 topological waves; 870 functions are leaves (wave 0)**, and
  **2,399 of 2,481 (97%) land by wave 12** — the last 14 waves are an 82-
  function integrator spine that ends at `Com_Init` (wave 26).
- **478 distinct externals** — names resolved outside the compile set: libc/
  std::, the `Sys_*`/`NET_*` platform seam (26 names), and the already-ported
  `q_shared`/`q_math` surface the Rust qshared crates supply.

Per-subsystem (fns / body-LOC / complete-at-wave / fn-statics):

| subsystem | fns | LOC | done at wave | fn-statics |
| --- | ---: | ---: | ---: | ---: |
| qcommon | 743 | 24,115 | 26 | 48 |
| botlib | 697 | 24,065 | 19 | 8 |
| server (incl. NPCNav) | 258 | 9,701 | 25 | 12 |
| icarus | 253 | 6,749 | 12 | 1 |
| ghoul2 | 224 | 9,230 | 19 | 40 |
| renderer (G2/model subset) | 148 | 9,900 | 13 | 9 |
| RMG | 113 | 3,856 | 16 | 1 |
| null (dedicated client stubs) | 45 | 112 | 6 | 0 |

**Cross-subsystem dependency matrix** (dependent → dependency, call + ref
edges):

| from \ on | qcommon | server | ghoul2 | botlib | icarus | RMG | rend | null |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| qcommon | *1233* | 12 | 1 | · | · | · | 4 | 27 |
| server | 410 | *382* | 50 | 1 | 18 | 6 | 5 | 2 |
| ghoul2 | 25 | 2 | *298* | · | · | · | 66 | · |
| botlib | 75 | · | · | *1578* | · | · | · | · |
| icarus | 25 | 6 | · | · | *436* | · | · | · |
| RMG | 87 | 1 | · | · | · | *85* | · | · |
| renderer | 93 | · | 11 | · | · | · | *140* | 1 |
| null | 1 | · | · | · | · | · | · | *·* |

Structural facts from the matrix (they order the progression; nothing is
deferrable off the back of them any more):

1. **qcommon is a true leaf.** 1,233 intra-subsystem edges and only ~44 out
   (12 `SV_*` callbacks from cm/event code, 27 into the null client stubs,
   4 renderer, 1 ghoul2). It is the floor of everything.
2. **botlib and icarus are near-isolates over qcommon.** botlib: 1,578
   internal, 75 into qcommon, zero elsewhere. icarus: 436 internal, 25 into
   qcommon, 6 into server. They complete early in the wave order (icarus by
   wave 12, botlib by 19) precisely because they depend on so little.
3. **ghoul2 ⇄ renderer are welded together** (66/11 edges both ways). Server-
   side Ghoul2 is not separable from the model/mesh/shader loader WinDed drags
   in from `renderer/` — they are one band of the progression.
4. **server is the integrator, not a mid-tier.** The corrected graph shows
   server depending on *everything*: 50 edges into ghoul2, 18 into icarus, 6
   into RMG, and 39 `SV_GameSystemCalls`→`CNavigator` calls that the old walk
   misclassified as externals (`server/NPCNav/` was missing from the compile
   set). `SV_GameSystemCalls` alone has 173 in-engine dependencies and sits at
   wave 20; `SV_SpawnServer` at 22; `Com_Init` closes the graph at 26.

Dispatch tables (indirect-call seams; the 261 address-taken edges order every
table target before its table builder):

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
- **File-scope fn-ptr arrays** (`ucmds[]` in sv_client.cpp and kin) register
  handlers from static initializers, not function bodies — those targets show
  up in the progression by their own dependencies, but their *reachability*
  is invisible to the call graph (see the ~688-function table/vtable bucket in
  §2).

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
  **This is the exact contract the real engine implementations must satisfy** —
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

**Class-A stubs to fill (current-tree scaffolding, already stubbed with the right
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
under engine/** — it lives in `crates/mp/renderer`; the ghoul2⇄renderer band
draws from there.

So: the **data model exists, the dispatch spine exists, the bodies do not.**
This plan is a body-filling plan — every body, in the total dependency order of
`engine-port-order`, gated by the referee that is already built. The `todo!`s
listed above are the current tree's scaffolding; the finished engine contains
none.

---

## 1. Scope — everything, 1:1, no stubs, no deferrals

The deliverable is a complete pure-Rust `openjkded`: it loads the existing game
dylib and answers its syscalls byte-identically to the C engine, with **every
function of the WinDed link set ported 1:1** — 110 sources, 2,481 functions,
87,728 LOC. No `todo!()` bodies ship, no subsystem is stubbed, deferred, or
cut (networking, bots, ICARUS, RMG, the VM interpreter/JIT all port). The only
two boundaries, both seams rather than deferrals:

- **The platform seam** (`null/win_main.cpp`, `win32/win_net.cpp`,
  `win32/win_shared.cpp` — the `Sys_*`/`NET_*` externals, 26 names): the Rust
  host implements these natively (std::net, std::time, the module loader that
  already exists). This is OS-API code, not game logic; 1:1 transcription of
  Win32 calls has no meaning on the target platforms.
- **Vendored third-party code** (`zlib32/`, `png/`): supplied by Rust crates
  (DEFLATE/PNG decoding is bit-deterministic), parity gated by pk3-checksum
  and terrain-heightmap golden fixtures. Consistent with the established
  vendored-code policy from the type port.

C-track functions transcribe faithfully (porting-rules §A–E). C++-track
subsystems — icarus, RMG, the ghoul2/renderer class internals — get the §F
design-first idiomatic shape, but **every method ports**; the only droppable
surface is zero-caller API per §20, recorded per module.

A consequence of no-stubs, straight from the graph: address-taken registration
(`Cmd_AddCommand` tables, the botlib/ICARUS dispatch tables) makes `Com_Init`'s
true transitive closure ~1,735 functions (70% of the engine, botlib included —
`SV_BotInitBotLib` fills `botlib_import_t` at boot). **A stub-free engine
binary first boots when nearly the whole progression has landed.** Capability
therefore arrives per-function and per-subsystem through the test harnesses
(§3) — oracle goldens, captured-replay fixtures, and referee swap-ins against
the mock — not through a runnable-but-stubbed binary. The mock engine is test
scaffolding on the harness side of the seam; the engine itself only ever
contains complete, verified functions.

### Port-process discipline (hard rules, user directive 2026-07-09)

- **No `todo!()`, no stub bodies, no `TODO`/`FIXME` markers at any commit
  during the port** — not just in the finished engine. The total order makes
  this achievable: a function is only ever ported after everything it
  references, so there is never a missing dependency to mark. The
  porting-rules unported-work marker scheme (`//TODO: Port <subject>`) is
  therefore **not used** in engine-port work at all; a porter who cannot
  complete a function without a placeholder stops and escalates instead of
  stubbing.
- **Greppable gate at every commit:** `grep -rn "TODO\|FIXME\|todo!"` over the
  engine crates stays empty (the existing `crates/mp/engine/` scaffolding
  `todo!`s are burned down as their waves land and may not be added to).
- **The order comes only from the tool.** `tools/closure-prototype/engineorder.py`
  is the single source of the port order; no hand-scheduling, no ad-hoc
  reordering. If the order looks wrong, fix the tool and regenerate — never
  the artifact by hand. It runs with the pinned parse configuration in the
  appendix; regenerating with different flags invalidates the artifact.
- **Transcription-first: no safety refactoring during the port.** Faithful
  port, parity green, refactor behind the passing diff (porting-rules §A2) —
  pointer-style Rust where the C demands it, exactly as the game module was
  landed. The safe-state migration is a separate post-parity phase, as it is
  for `mp_game`. The §3 heading "build it safe" means the three structural
  conventions decided at port time — globals → host-struct fields (never
  `static mut`), §F design-first shape for the C++-track subsystems, `unsafe`
  confined to the ABI seam — not safety idiom conversion.

### Stage 0 (prerequisite, from the referee plan) — the game-host interface crate
Not engine code; the contract. A `crates/mp/host-interface` (name TBD) crate of
Rust traits transcribing the C seam: the **syscall surface** (the `SV_*`/`CM_*`/
`FS_*` functions the game reaches through `trap_*` = the `MpGameImport` numbers in
`crates/mp/abi`), the **vmcall driver** (`GAME_INIT`/`GAME_RUN_FRAME`/
`GAME_CLIENT_THINK`/... via the existing `RawVmMain` 12-word trampoline), and the
**shared-memory contract** (`G_LOCATE_GAME_DATA`, the `gentity_t`/`gclient_t`/
`playerState_t` stride agreement already asserted in the port). Both the referee
harness and every real engine crate implement/consume this trait — this is the
"one marshaling dispatcher" (referee-plan task #9) realized once. **Build this
before wave 0.** Two working seeds converge here: the **mock engine**
(`crates/jampgame/tests/common/mod.rs`) is the behavioral spec of exactly this
trait, and the **dispatch spine** already exists (`arm_game_slot` +
`game_system_calls_shim` + `sv_game_system_calls` in `server_host.rs`). Stage 0
is extracting the mock's arms into a real trait and making
`sv_game_system_calls` dispatch it with the real `ServerGame` ctx (today it
injects `null_mut()` and `todo!`s every arm but `G_PRINT`).

### The bands of the progression (descriptive, not deferrable)

The wave order interleaves subsystems; these bands describe what the work *is*
in each region of the progression, with the design dossiers that feed it. They
are reading order, not an option menu — every band ports.

- **The util floor and the core knot** (waves 0–6 carry 1,974 functions, 80%
  of the engine): pure math/string/parse leaves, `md4`/`crc`, `huffman.cpp`
  (the `msgHuff` 102KB table), `msg.cpp` delta encode/decode, the cm_* trace
  pipeline, unzip/pk3, and — as one 110-function cyclic unit at wave 5 — the
  qcommon core knot (`Com_Error` ⇄ `Com_Printf` ⇄ `Cbuf`/`Cmd` ⇄ `Cvar` ⇄
  `FS_*` ⇄ `CM_*`: error handling calls back into everything, so the core
  lands together). Also the whole `null/` client-stub layer (45 tiny fns,
  done at wave 6). Dossiers: `docs/dossiers/B1-cvar-cmd.md`,
  `docs/dossiers/B2-filesystem.md`, `docs/dossiers/B3-collision.md`.
  Known-high-risk parity targets living here: `sv_world.cpp` entity linking
  (the areagrid — the Session-2 "invisible elevator" link→absmin→box-query
  seam has an existing reproducer), `CM_Trace`/`CM_LoadMap_Actual` (carries
  the `last_checksum` determinism static), `FS_FOpenFileRead` (324 LOC).
- **The VM subsystem** (`vm.cpp`, `vm_interpreted.cpp`, `vm_x86.cpp`): ports
  in full — no "native-dylib-only" carve-out. `VM_Call`/`vmMain` trampoline
  marshaling is the live path for our game dylib; the bytecode interpreter and
  the x86 JIT emitter port 1:1 alongside it (the JIT executes only on x86
  hosts, exactly as in the C engine — behavior parity, not a stub; see §5.4).
- **icarus** (253 fns, done at wave 12): ROFF/script sequencer +
  `Q3_Interface`/`GameInterface` — C++ class tree, §F design-first. Note the
  WinDed set does **not** link `icarus/Interpreter.cpp`/`Tokenizer.cpp` (the
  old 540-fn count included them); the linked sequencer is 6.7k LOC.
- **ghoul2 ⇄ renderer** (224 + 148 fns, done at waves 19/13): server-side
  bone/bolt/collision (`G2_RagDoll*` 485/417/286, `G2_TransformBone`) welded
  to the headless model/mesh/shader loader (`.md3`/`.glm`/`.mdx` parsing;
  `ParseStage` 642 / `FinishShader` 335 are shader-text parsing, not GL).
  Ghoul2 carries **40 function-scope statics** (the RagDoll solver is riddled
  with them) — the highest-effort-per-LOC band.
- **botlib** (697 fns, done at wave 19): AAS route/reachability/movement, the
  `.aas` binary loader, and the import/export tables. On the boot path
  (`Com_Init` → `SV_BotInitBotLib`), not an add-on.
- **RMG** (113 fns, done at wave 16): random-mission terrain — C++ class tree,
  §F design-first. In the link set, referenced by the syscall switch (6
  server→RMG edges); ports like everything else.
- **The server integrator and the wire** (server done at wave 25): snapshot/
  client path (`SV_BuildClientSnapshot`, `SV_DirectConnect` 348,
  `SV_ExecuteClientMessage`), `net_chan.cpp`, `NPCNav/` (CNavigator — 39
  direct callees of the syscall switch), `SV_GameSystemCalls` (wave 20, 173
  in-engine deps), `SV_SpawnServer` (wave 22), and the `Com_Frame`/`Com_Init`
  spine closing at wave 26. SnapVector/`MSG_WriteDeltaPlayerstate`
  byte-faithfulness lessons from the handoff are load-bearing here. Dossiers:
  `docs/dossiers/B4-network.md`, `docs/dossiers/B5-server.md`.

---

## 2. Port order (the total progression)

`engine-port-order.{json,tsv,md}` **is** the port order: one row per function,
`seq` ascending. Port in that order and no symbol is ever referenced before it
is ported — machine-verified at generation time (every dependency edge lands
in an earlier wave or inside the same cyclic unit). Rules:

1. **The artifact is the manifest.** No hand-scheduling across subsystems; the
   wave field encodes everything the old tier ordering encoded, corrected by
   the true graph (e.g. server is the integrator — it depends on ghoul2,
   icarus, RMG, and CNavigator, so its top lands last; see §0).
2. **Waves fan out; the spine serializes.** Wave 0 is 870 functions and
   embarrassingly parallel — the same "port-wave" agent fan-out that landed
   the type waves (NOTES v8–v11) applies, keyed on the artifact's `wave`
   field. 97% of the engine lands by wave 12; the final 82-function spine
   (waves 13–26) is integration work and lands serially.
3. **Cyclic units port as one unit.** There are 7 in the whole engine. The
   largest (110 functions, wave 5) is the qcommon core knot — `Com_Error`/
   `Com_Printf`/`Cbuf`/`Cmd`/`Cvar`/`FS`/`CM` mutual recursion — and lands as
   one coordinated unit; the rest are small. Enumerated in the artifact
   (`cyclic: true`).
4. **Statics and mutable globals are explicit port tasks, not incidental.**
   Each of the 119 function-scope statics is a hidden persistent cell (e.g.
   `CM_LoadMap_Actual::last_checksum`, the botlib `AAS_ContinueInitReachability`
   frame counters, Ghoul2's RagDoll `static` scratch). Each must be threaded
   through explicit state (a host struct field), never a Rust `static mut`, or
   determinism and reentrancy break. The ~680 file-scope globals (`sv` 665KB,
   `tr` 316KB, `msgHuff` 102KB, `cvar_indexes`, the `fs_server*` pak arrays)
   are the shared-state struct fields of the host — they define the host's
   data model and should be laid out before their functions are ported
   (census: `engine-fn-stats.md`).
5. **The table/vtable bucket is in the order, audited for deadness.** ~688
   functions are not statically reachable from the boot/frame/syscall roots —
   they are reached through file-scope fn-ptr arrays (`ucmds[]`), populated
   dispatch structs, and C++ virtual dispatch, or they are genuinely dead in
   the dedicated build. All of them are in the progression (no reachability
   deferrals); the per-file oracle review marks any true dead code with a §20
   zero-caller note instead of porting it speculatively.

---

## 3. Safety / testing methodology — "build it safe, test it to the oracle"

The port already has a proven parity discipline for the game module; this plan
reuses all four layers of it, aimed at engine functions. The ordering principle:
**pure functions get golden fixtures; stateful subsystems get captured-replay;
whole subsystems get engine-vs-engine lockstep as they complete.**

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
cm and sv_world seams (the two highest-risk parity targets) at scale.

### 3c. Subsystem swap-in acceptance (A/B lockstep)
The gate at each milestone. Two A/B mechanisms exist, and the plan leans on the
one that is actually alive:
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
  but its worktree (`scratchpad/openjk-seam`) was session-ephemeral. As of
  2026-07-08 it has been reconstituted into a permanent home — see risk #7 —
  so this path can gate work again; the in-repo Rust referee remains the gate
  for anything not yet covered by the reconstituted rig.

Concretely: as each subsystem lands it is swapped into the Rust host while the
rest stay trusted (mock, or later real C engine), and the A/B must be byte-
identical before the next subsystem begins.

Two asymmetries of the external rig to keep in view (from the build-flag
comparison, 2026-07-09): the OpenJK referee engine builds Release with
**`FINAL_BUILD` defined**, while this port (matching WinDed vcproj Release and
the game module's settled convention) compiles the non-FINAL_BUILD dev paths —
corpora must not exercise dev-only paths, or the OpenJK side needs a
non-FINAL rebuild for those runs. And OpenJK is **different source** (files
renamed/merged, q_shared modernized, RMG dropped entirely), so engine-vs-
engine A/B is a behavioral gate at the seam, never line-for-line; anything
touching the 6 server→RMG syscall edges verifies only against oracle-derived
goldens (3a/3b), not against OpenJK.

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

Milestones are wave-prefix checkpoints of the one progression — each is
dependency-closed by construction, so nothing behind a checkpoint references
anything ahead of it. "Unlock" is what the harnesses can newly verify; the
engine binary itself first boots at M5 (see §1 — no-stub boot needs the whole
graph).

| # | milestone (wave prefix) | fns landed (cum) | complete there | unlock |
| --- | --- | --- | --- | --- |
| M0 | interface crate + vmMain trampoline; game loads into a Rust process that immediately hands every syscall back to the C engine | — | Stage 0 | the seam is Rust-owned; A/B harness has a trait to swap against |
| M1 | waves 0–6: util floor, msg/huffman/cm/pk3, the 110-fn core knot, null layer | 1,974 (80%) | null (w6); the qcommon core | oracle goldens (3a) green for msg/huff/md4/q_math-class leaves; replay fixtures (3b) run against Rust `CM_Trace`/`SV_LinkEntity`-class functions; mock arms for FS/cvar/configstrings swap to real implementations in the in-repo referee |
| M2 | waves 7–12: the parallel bulk | 2,399 (97%) | icarus (w12) | icarus sequencer differential-golden (§F pattern, `tools/<subsystem>-oracle/`); cm/sv_world seams fully real under the referee — the invisible-elevator link→box-query seam owned in Rust |
| M3 | waves 13–19: subsystem tops | 2,452 | renderer (w13), RMG (w16), botlib + ghoul2 (w19) | G2 bone/bolt/collision goldens; `.aas` load parity; every mock arm except the server spine replaced by real code |
| M4 | waves 20–25: the integrator | 2,480 | `SV_GameSystemCalls` (w20), `SV_SpawnServer` (w22), server (w25) | every syscall arm is real Rust dispatched through the interface trait; in-repo referee drives the real server spine under `cargo test` |
| M5 | wave 26: `Com_Init` | 2,481 (all) | qcommon; the binary | **pure-Rust `openjkded` boots**; external engine-vs-engine A/B on corpus-ffa1/-combat; **`taystjk` connects over the network**; the original prediction-miss class re-tested end-to-end on Rust host + Rust game |

Each milestone's gate is green oracle goldens (3a) and replay fixtures (3b)
for everything in the prefix, plus the referee swap-in (3c) for every
subsystem completed by that wave. M5's gate is the external lockstep rig
(3c-external) — the same discipline that verified the game port.

---

## 5. Risks and open questions (specific, from the walk)

1. **C++ subsystems in a C-centric pipeline (icarus, RMG, ghoul2/renderer
   class internals).** The port pipeline (closure/portpacket/fnskel) is tuned
   for free C functions. The graph now captures method→method calls (RMG shows
   its real 85 internal edges; the old walk showed 0), but **virtual dispatch
   still resolves edges to the statically-named method** — overrides reached
   only through a vtable have no incoming call edge and sit in the ~688-fn
   table/vtable bucket (§2.5). Consequence: these subsystems follow the §F
   design-first track (closed hierarchies → enums, arena/handle shapes decided
   once, then methods transcribed into the shape) with differential goldens
   (`tools/<subsystem>-oracle/`), not blind packet transcription. Every method
   still ports; §F governs *shape*, not scope.
2. **botlib's AAS binary file formats.** botlib is 24k LOC and its determinism
   hinges on loading `.aas` area-awareness files (`be_aas_file.cpp`, the 78
   `LittleLong` byte-swap sites the sweep flagged). Parity requires bit-exact
   `.aas` parsing and the same route-cache behavior; the `AAS_ContinueInit*`
   frame-spread statics make it stateful across frames. High effort, and **on
   the boot path** (`Com_Init` → `SV_BotInitBotLib` fills `botlib_import_t`) —
   plan the fixture harness early, not as an afterthought.
3. **Ghoul2 ⇄ renderer entanglement is structural, not incidental.** The 66
   ghoul2→renderer edges are real: `tr_ghoul2.cpp` (`G2_TransformBone` 511) lives
   in `renderer/` but is pure bone math the *server* needs for collision. WinDed
   compiles the real `tr_model`/`tr_mesh`/`tr_shader` under `DEDICATED`, so "the
   dedicated server has no renderer" is false — it has a headless model/shader
   loader. The boundary between "model data the server needs" and "GL drawing
   the server doesn't" runs *through* individual files via `#ifndef DEDICATED`,
   not along file lines. Budget for reading those ifdefs carefully.
4. **The VM subsystem ports whole; the trampoline ABI is the live wire.**
   `vm.cpp`/`vm_interpreted.cpp`/`vm_x86.cpp` are in the WinDed link set and in
   the progression — `VM_Compile` (698) and `VM_CallInterpreted` (610) are the
   two largest qcommon functions. The x86 JIT ports 1:1 as emitter logic but
   can only *execute* on x86 hosts (identical to the C engine's behavior on
   arm64 — a runtime dispatch fact, not a stub). At runtime our game module
   loads as a native dylib, so the hot path is the `vmMain`/
   `SV_GameSystemCalls` marshaling, which must match the C ABI exactly,
   including the `intptr_t`-through-arg-slot widening the oracle build needed
   (handoff: `GAME_NAV_CLEARPATHTOPOINT` truncated a stack pointer). The
   interface crate must use `intptr_t`-width slots, not `int`.
5. **Parse coverage caveats (report them, don't trust silently).** The
   original unity-TU walk had three silent defects, all fixed in
   `engineorder.py`: (a) whole statement runs dropped with **zero
   diagnostics** (`sv_game.cpp`'s `VMA()` cases — `SV_GameSystemCalls` showed
   36 callees instead of 173; `msg.cpp`'s netField tables gutted by LP64
   pointer-cast errors, rescued with `-fms-compatibility`); (b) C++ method
   calls and address-taken references were not edges; (c) the compile set was
   a glob, not the vcproj — it missed `server/NPCNav/` and `null/` and swept
   in 13 never-linked xbox/console/ppc files. Residual diagnostics in the
   per-file parse are enumerated in the artifact's `diag_files_top` and are
   the known-benign classes (SnapVector MS-asm on arm64, libc++ noise from
   `-fno-operator-names`, ~26 each in `tr_ghoul2`/`tr_model`, 25 in
   `files_pc`) — **re-verify per-file when packets are generated** for those
   files. Type layouts still come from the separate layout pipeline, never
   from this walk.
6. **Open: where does the interface crate's boundary sit** relative to the
   existing `crates/mp/engine/qcommon` trampoline seed — extend that crate or
   supersede it? (Resolve against the seeds map before M0.)
7. **RESOLVED 2026-07-08: the external openjkded rig has a permanent home.**
   The engine-vs-engine lockstep (3c, richer path) and the captured trace
   fixtures (3b) depend on the patched `openjkded`/`sv_referee.cpp`, previously
   in the session-ephemeral `scratchpad/openjk-seam`. The patch series is now
   maintained at github.com/mheh/OpenJK branch `referee` (commit f6d2875e
   "sv_referee: A/B module-parity referee layer" + 35e4184f temporary debug
   probes), cloned at ~/Developer/Milo/OpenJK. Phase 0 (rebuild + re-verify)
   completed 2026-07-08: engine rebuilt via `cmake -B build-referee`
   (BuildMPDed only); corpus-ffa1 PASS 4000 frames and corpus-ffa1-combat PASS
   5000 frames, zero divergence, oracle-built Raven dylib vs Rust jampgame
   dylib. `~/Developer/jka/seam-test/referee/run-ab.sh` now points ENGINE at
   the from-source build; the old preserved binary is retired. 3b/3c-external
   can gate work again; scenario-matrix expansion beyond the two re-verified
   corpora remains open before the M5 real-network acceptance.

---

## Appendix — reproducing the walk

```
cd tools/closure-prototype
.venv/bin/python engineorder.py            # -> out/engine/engine-port-order.{json,tsv,md}  (the port order; ~3 min)
.venv/bin/python enginesweep.py            # -> out/engine/{engine-fn-manifest.json, engine-fn-stats.md}  (statics/globals census only; graph superseded)
.venv/bin/python closure.py --list-modules # mp-engine-ded profile listed under [raven]
```

`engineorder.py` derives its source list from `oracle/codemp/WinDed.vcproj`
directly (minus the platform seam and vendored zlib/png), parses one TU per
source file, merges the graph by USR, and **asserts the no-stub property** of
the emitted order — see its docstring for the four defects this fixes over the
unity-TU approach. `enginesweep.py` remains for the per-subsystem LOC
histograms and the statics/globals census. All of it is throwaway prototype
tooling per the port-tooling principle (oracle → self-contained,
machine-verifiable work orders).

**Pinned parse configuration** (user directive 2026-07-09 — the artifact is
only valid under these flags; `engineorder.py` applies them on top of the
`mp-engine-ded` profile):

- Macro set = WinDed.vcproj Release: `-DNDEBUG -DDEDICATED -DBOTLIB`;
  **`FINAL_BUILD` undefined** (matches vcproj Release and the game module's
  settled non-FINAL_BUILD convention); **no platform macro** (`WIN32`/
  `MACOS_X` omitted — 200 of the 215 platform-gated regions in the set are
  `_XBOX`, off in every PC build; the 16 WIN32-gated sites in 7 files get a
  per-site read when their packets are cut).
- Parse shims that create/remove no Raven code: `-fdeclspec`
  `-fno-operator-names` `-fms-compatibility` (downgrades the MSVC-era
  pointer-truncation casts that otherwise silently drop `msg.cpp`/`sv_game.cpp`
  statements), the win32 typedef defines, `stricmp=strcasecmp`, and
  `LittleShort= LittleLong= LittleFloat= LittleLong64=` (little-endian
  identity, mirroring Raven's own WIN32 branch).
- These are **parse** flags for graph extraction. Runtime float parity keeps
  its own settled recipe (oracle reference built `-fno-fast-math
  -ffp-contract=off -fsigned-char`, §3a); the two configurations serve
  different layers and are not interchangeable.
