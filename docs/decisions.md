# Decision Ledger

Architectural decisions settled with the user (2026-07-01 session) that design docs
cite but never re-litigate. Design docs reference entries by ID (`DEC-xx`).
Amendments are appended as dated notes — entries are never silently rewritten.

Per-doc design decisions (`<PREFIX>-Dn`) live in each design doc's `## Decisions`
section; this ledger holds only cross-cutting choices.

---

## DEC-01 — Renderer port deferred; wgpu when it happens

The renderer is **not ported until much later** in the project. Until then the
engine boots headless: MP dedicated uses Raven's own null path
(`oracle/codemp/null/`), and MP client / SP get a **null-renderer
`refexport_t` stub** behind the same seam (renderer types are already ported,
Wave 7). When the renderer is ported, **wgpu is the likely backend** (an
idiomatic C++-track-style rewrite, not a fixed-function GL transcription) —
re-confirm specifics at that time.

Implications: `docs/subsystems/renderer.md` covers the seam + null-renderer
deferral strategy only; lifecycle docs must define headless boot for all three
executables.

> **Amendment 2026-07-02:** the earlier note that SP boots the renderer early
> inside `Com_Init` was wrong — that block
> (`oracle/code/qcommon/common.cpp:965-981`) is `#ifdef _XBOX` dead
> code. On PC, SP initializes the renderer inside `CL_Init` like MP (A3
> survey). Headless design needs no special SP early-boot handling.

## DEC-02 — Windowing/input: winit

`native/platform` uses **winit** (+ `raw-window-handle`) for window and input.
The platform doc defines the adapter between winit's event-loop model and
Raven's poll-style `Sys_*`/input pump.

> **Amendment 2026-07-02:** accepted divergence — under winit's `Poll` mode,
> OS input arriving during the FPS-cap busy-spin is deferred one frame
> (≤`1000/com_maxfps` ms; Raven pumps `PeekMessage` per spin iteration,
> `oracle/codemp/qcommon/common.cpp:1647-1653` →
> `oracle/codemp/win32/win_main.cpp:1211`). This sits below the
> differential seam (journaling feeds the event ring directly). Recorded in
> `lifecycle.md` LIFE-D1.

## DEC-03 — Audio: cpal + faithful mixer

Raven's software mixer (`snd_dma`/`snd_mix`) is ported **faithfully** and outputs
through **cpal**. MP3 decode via `minimp3` (existing vendored-replacement
policy); EAX and force-feedback stay dropped. Keeps mixing behavior
parity-testable at the sample level.

## DEC-04 — Cross-mode sharing: strict per-mode during porting

Every Raven-derived engine subsystem is **duplicated per-mode** during the port —
porters never face a sharing decision, each crate diffs against exactly one
oracle subtree, and crate boundaries stay 1:1 with Raven compile-lists.
Unification of a provably-identical MP/SP pair is allowed **only as a
post-parity refactor** behind green differential tests (porting-rules §A2).
`native/` remains math/platform/containers only.

## DEC-05 — Module loading scope (and the WASM transport)

In scope:

1. **Rust engine ↔ Rust modules** — core scenario, all platforms, via
   `native/platform` dylib loading honoring Raven's entry symbols.
2. **Rust modules loaded by real engines** — retail `jamp` 1.01 (requires
   `i686-pc-windows` module builds) and OpenJK native builds (account for the
   Raven-vs-OpenJK ABI divergences documented in
   `tools/closure-prototype/NOTES.md`).
3. **Rust engine hosting real/mod DLLs** (`jampgamex86.dll` replacements: JA+,
   MBII, …) — **`i686-pc-windows` engine build only** (retail DLLs are 32-bit
   Windows PE). Mod-ecosystem compatibility is the goal that justifies this.
4. Classic QVM bytecode interpreter: **out of scope** (JKA shipped native DLLs).
5. **WASM module transport: first-class target from the start.** The module
   transport is pluggable — `NativeDll | Static | Wasm` — and
   `architecture/engine-seam.md` + `architecture/module-loading.md` must design
   the WASM variant explicitly (wasm32 linear-memory pointer translation à la
   `VM_ArgPtr`, handle-only trap surface, 32-bit in-module layouts). Module
   crates get `wasm32` build checks in CI from the beginning so portability
   never regresses. The wasmtime host itself lands **after** native-DLL parity
   is proven (a sandbox is no place to debug parity).

**AMENDED 2026-07-09 (ruling 35, engine-fork-discovery):** item 5 **REVERSED** —
WASM dropped entirely (no module transport, no engine target, no scripting
sandbox); `Wasm` variant, `WasmPtr`, `wasm-host` crate, and wasm32 CI ambitions
all removed. Transports: `NativeDll | Static`.

## DEC-06 — Network: full 1.01 wire compatibility

Exact protocol 26 on the wire — huffman coding, delta snapshots,
challenge/connect flow — so Rust peers interoperate with real 1.01/OpenJK peers.
Live interop doubles as the strongest oracle (see DEC-09 layer 2), and it is
what "drop-in `jampded`" means.

## DEC-07 — SP cgame/ui: statically linked via the vmachine shim

`sp/app` statically links `sp/cgame` + `sp/ui`; Raven's fake-VM shim
(`oracle/code/client/vmachine.cpp`) survives as a thin dispatch layer
preserving the `VM_Call` ABI shape — matching shipped `jasp`. The retail
load-from-DLL variant is not ported. Resolves workspace-architecture's "SP
transport" open item.

> **Amendment 2026-07-02:** the vmachine shim is preserved as the **inbound**
> `VM_Call`-shaped dispatch surface into cgame's `vmMain` only; outbound
> cgame→engine calls are direct typed calls through the `Static` transport (no
> word packing — `oracle/code/client/vmachine.cpp:36-39`'s
> `VM_DllSyscall`→`CL_CgameSystemCalls` round-trip is internal plumbing with no
> observable behavior). Recorded in `engine-seam.md` SEAM-D1.

## DEC-08 — Com_Error recovery: panic + catch_unwind

`Com_Error(ERR_DROP, …)`'s dynamic escape to `Com_Frame` becomes a **typed
panic payload** caught by `catch_unwind` at the frame boundary, running the
same recovery Raven does. Requires `panic = "unwind"` and a per-subsystem audit
that unwound state resets match Raven's error paths. Result-threading was
rejected: it reshapes thousands of faithful signatures away from oracle
control flow.

> **Amendment 2026-07-02:** Raven's mechanism is C++ `throw`/`catch` (string
> exceptions like `"DROPPED\n"` caught in `Com_Frame` —
> `oracle/codemp/qcommon/common.cpp:1762`,
> `oracle/code/qcommon/common.cpp:1450`), not setjmp/longjmp as first
> recorded. Recovery runs *before* the throw; the catch prints and returns.
> This maps onto panic+catch_unwind even more directly — the decision stands
> unchanged. (A3 survey.)

## DEC-09 — Engine verification: TU harnesses + live peers

Two layers, used by every subsystem doc's `## Verification strategy`:

1. **TU-level golden harnesses** (the `tools/gp2-oracle` pattern): compile the
   unmodified oracle TU standalone with stub headers, golden-diff canonical
   dumps against the Rust port — for huffman, msg pack/unpack, cvar parsing,
   netchan framing, and anything else that runs standalone.
2. **Live-peer slice acceptance** (per DEC-06): scripted sessions between the
   Rust engine and real binaries (retail/OpenJK), diffing observable behavior —
   console output, cvar dumps, wire traffic captures.

Building the full Raven engine from oracle source was rejected as a project in
itself; OpenJK serves as the buildable near-oracle where a live peer is needed.

## DEC-10 — Incremental builds with checkpoint/reset semantics

Validation builds for the logic port are **cumulative, not scratch**
(2026-07-03 session):

- A persistent **skeleton branch** is seeded from the settled design surface —
  crates, frozen signatures, `todo!()` bodies. `cargo check` green on the seed
  is the base build; rustc replaces prose probes as the dry-run referee
  (SEAM-Q12-class contradictions become compile errors).
- Every phase/agent builds on top; **each green state is a checkpoint commit**
  (tag per phase boundary).
- **Reset semantics:** minor issue observed while building → adjust (doc
  amendment and/or skeleton fix) and continue; foundational issue → `git reset`
  to an earlier checkpoint and fix the foundation; unrecoverable → re-branch
  from the seed commit. Nothing is re-derived from scratch.
- The validated skeleton **is** slice 0's starting point, not throwaway gate
  output; the `port-slice` workflow builds on it.

> **Amendment 2026-07-03 (same session):** applied **immediately**, not after
> FROZEN ×4 — the baseline is seeded in parallel with the round-4 doc pass from
> the user-settled resolutions (all 29 forks closed). Round-4 doc deltas
> reconcile into the skeleton as minor fixes per the semantics above.

## DEC-11 — Post-parity seam inversion (end-state, not pursued yet)

Once oracle parity is proven per subsystem, the generic transport layer
(`Execute<C>`/`Dispatch<C>`, typed calls) becomes the **true call path**
wherever both endpoints are ours; the Raven ABI shapes solidify into a frozen,
layout-asserted **compatibility shell** engaged only at foreign-endpoint edges
(original DLLs, real engines, wire peers). The shell gets first-class dual-path
CI (our engine + retail DLLs, our DLLs + OpenJK, live-peer wire sessions per
DEC-09) so it never rots.

Standing priority during the port (user, 2026-07-03): **possibility over
pursuit** — keep the inversion possible, change nothing toward it now; syntax
and semantics of Raven behavior are what matter most during this process.
Sequenced strictly after per-subsystem parity (porting-rules §A2's
faithful-first rule); fits DEC-04's arc (unify only after porting). Precedents
already inside the settled decisions: DEC-07's dropped word-packing round-trip
(internal plumbing, no observable behavior) and DEC-05's since-reversed wasm
transport (the generic layer is already the true call in one mode).

## DEC-12 — Ctx-free boundaries route via narrow capability sinks (user, 2026-07-06)

Raven's two ctx-free fn-pointer boundaries — `Com_Printf`/`Com_Error`
(`g_main.c:1208-1228`), called from bg-tier sites that carry no
`GameContext` by design — reach the engine through **print-only fn-pointer
sinks**: `OnceLock<fn(*const c_char)>` statics in `mp_game`
(`com_boundary.rs`), registered by the shell at `dllEntry` with fns that
route through its sanctioned `ENGINE` static to `trap::Printf`/`trap::Error`.
Ruled a narrow SEAM-D1 extension after weighing three options:

- **Chosen — capability sink**: the ambient channel can *only* print;
  widening it is a visible type change requiring a new ruling. Matches the
  future bg-split shape ("bg needs a print capability") so it dissolves
  cleanly at Stage 1/2.
- Rejected — whole `OnceLock<Engine>` in `mp_game`: faithful to Raven's
  g_syscalls.c file-static, but makes the entire syscall surface ambiently
  reachable — a standing temptation to erode porting-rules §B4.
- Rejected (for now) — threading `&Engine` into all ~32 calling files: the
  §B4-pure end state, but forces the bg-tier capability design mid-referee
  and churns bg signatures the future `mp_bg`/cgame split owns.

Fallbacks when unregistered (in-process tests without `dllEntry`):
`Com_Printf` keeps `eprint!`; `Com_Error` keeps the frozen Group A `panic!`.
On the registered path `Com_Error` forwards and returns, dropping `level`,
exactly as Raven's body does. GameWorld is untouched — no world state goes
ambient; the two statics hold immutable fn pointers set once.

## DEC-13 — Botlib export tables → receiver-carrying fn-ptr structs; import stays C-shaped (2026-07-11)

The botlib **export** tables (the `botlib_export_t` surface the engine calls into)
are retyped as engine-internal receiver-carrying function-pointer structs — each
slot threads the botlib receiver instead of Raven's ambient `void(*)()` — while the
botlib **import** table (the services botlib calls back through) stays C-shaped in
`mp_qshared`. Commits `871e168b` / `cbd8388f`.

Rationale: the export side is reached only by our engine, so it is free to carry the
receiver (porting-rules §B4, state threaded not reached); the import side is the
foreign-facing seam botlib fills, so it keeps Raven's layout.

## DEC-14 — Console-command table slots → receiver-threaded fn pointers (2026-07-11, extended 2026-07-12)

Raven's console-command handlers (`void(*)(void)`) become table slots of
receiver-threaded function pointers; command dispatch threads the pinned receiver
set into each handler rather than reaching ambient `level`/engine state. Extended to
the RMG and ghoul2 (`g2`) command registrations 2026-07-12. Commits `ee62d80b` /
`f7cbb74d` / `2f355e97`.

Rationale: keeps the table-driven dispatch shape while honoring §B4 — a console
handler that mutates the world receives it, never reaches it.

## DEC-15 — EngineHooks upcall table on Common resolves the qcommon→server/client seam (2026-07-12)

The `qcommon`→`server`/`client` link seam (Raven's direct cross-module calls the
linker resolved) is modeled as an **EngineHooks upcall table owned by `Common`**. The
dedicated build installs WinDed's null link set as the defaults (mirroring
`null_client`/`null_snddma`); the `SV_*` server hooks and the renderer hooks are
**mandatory** — no null default, a missing install is a bug, not a silent no-op.
Commit `7c31900b`.

> **AMENDS DEC-01's framing:** WinDed also links the **real** `tr_model.cpp`, so on the
> dedicated build `RE_RegisterModels_LevelLoadEnd` / `R_HunkClearCrap` are **not** null
> stubs — they have real bodies (now in `mp_renderer`), not the null-renderer no-ops
> DEC-01's headless-boot framing implied. The renderer hooks are mandatory for exactly
> this reason.

## DEC-16 — Above-tier receivers are type-erased opaque slots in qcommon (ruling A, 2026-07-12)

The receivers that live *above* the `qcommon` tier — `Server`, `Client`, `BotLib`,
`Ghoul2System`, `RmManager`, `RenderModels` — are held in `qcommon` as **type-erased
opaque slots** (pass-through proven: qcommon never names their types, only stores and
forwards them), cast back to the concrete type at the owning crate's boundary through
**one documented `unsafe` pair** (erase at install, recover at use). Commits
`633d7291` / `cbd8388f` / `0530967c`.

Rationale: qcommon is below these crates in the graph and cannot name their types; the
opaque slot keeps the dependency acyclic while the single cast pair confines the unsafe
to the seam (§D11).

## DEC-17 — zlib via flate2 both directions; minizip parsed faithfully (2026-07-11/12)

zlib is backed by **flate2** in both directions: raw inflate for pk3 entry reads, and
zlib-framed deflate with `Z_SYNC_FLUSH` for the download stream (the deflate seam
landed for `sv_client` `:768-803`). The minizip container format is parsed
**faithfully** in Rust, not delegated to a minizip binding. Commits `d7706fab` /
`07974910`.

Rationale: flate2 is the sanctioned vendored-replacement for the codec (DEC-03-class
policy); the container parse stays ours so pk3 iteration/order matches Raven
byte-for-byte.

## DEC-18 — Server skins are a name-only pool; R_GetSkinByHandle is the host accessor (2026-07-12)

On the dedicated build the skin system is a **name-only pool**: `RenderModels` owns
`tr.skins`/`numSkins`, and a skin surface's `shader` is resolved by **name only** — the
dedicated path reads exclusively `shader->name` (`G2_surfaces.cpp:212`), never a
compiled shader. `R_GetSkinByHandle` is exposed as an `EngineHost` accessor so
`mp_engine_ghoul2` reaches skins across the service seam without an `mp_renderer` edge.
Amends `docs/subsystems/tr-model.md` and `docs/subsystems/ghoul2-server.md` (dated
amendments). Commit `64a48bb8`.

Rationale: the DEDICATED `refexport_t` carries no shader-compile surface (DEC-01); the
server needs only the surface→shader-name mapping, so the pool carries names — closing
ghoul2's second loader model-memory gap (the sibling of the `model_mdxm`/`model_mdxa`
block read).

## DEC-19 — q_shared.c shared families are module-island duplicates at the shared tier (2026-07-12)

The `q_shared.c` string / format / parse / math / swap families live at the **shared
tier** (`mp_qshared`) as **module-island duplicates** of `mp_game`'s already-verified
copies — the two are not unified (DEC-04, duplicate-don't-unify), each island carrying
its own §20 function-scope statics. Commits `9f074a04` / `46daed79`.

Rationale: engine crates below `mp_game` need these helpers but cannot depend on the
game module; duplicating the verified copy at the shared tier keeps the graph acyclic
and each island diffable against the one oracle TU.

## DEC-20 — Extern forward-declaration blocks are BANNED (the closure-sweep convention, 2026-07-12)

Extern / forward-declaration blocks that paper over an unported callee are **banned**.
An unported callee is left as an honest unresolved reference — a real `E0425`/`E0432`
at the callee's canonical home — so the remaining work surfaces as a compile error at
the right location rather than a silent forward decl. Commits `54fe7d6c` / `55904017`
/ `082e89b5`.

Rationale: porting-rules §E14 (unported deps explicit, never a silent fake); a forward
decl is exactly the silent fake the marker convention forbids.

## DEC-21 — The BOTLIB import slot is the state-capture dual of GAME_SLOT (SEAM-D11, 2026-07-11)

The `BOTLIB` import slot captures the botlib receiver the way `GAME_SLOT` captures the
game module's — it is the **state-capture dual of `GAME_SLOT`** (SEAM-D11): the shell
pins the receiver at install and every botlib import dispatch threads it, mirroring the
game-module boundary. Commit `cbd8388f`.

## DEC-22 — Platform seams: native Sys_*, engine-state wrappers in qcommon, threaded net/event receivers (2026-07-12)

`native_platform` implements the `Sys_*` surface **natively** in Rust (against the host
OS) rather than transcribing Raven's unix oracle; the `Sys_*` wrappers that reach engine
state live in `qcommon`'s `sys_engine` module; and the net/event receivers are
**threaded** — no `PlatformHost` global (§B3). Commits `e5977d43` / `d8111bd3` /
`e5731cfb`.

Rationale: the platform layer is the one place a faithful transcription buys nothing
(the oracle is unix-specific); native Rust is cleaner and cross-platform, while
engine-reaching wrappers stay in qcommon so no platform global is introduced.

## DEC-23 — The host seam is `EngineHostView`: one borrowed world bundle, not receiver lists beside a dyn host (2026-07-11)

The engine island's live `EngineHost` is `EngineHostView<'a>` in
`mp_engine_qcommon` (`common/engine_host_view.rs`): `{ &mut Common, &mut
CollisionWorld }` plus the six opaque slots (`sv`/`cl`/`bot`/`rm`/`rmg`/`g2`),
implementing the trait by routing back through the view-migrated functions
(`self` recursion) and, for `Server`/`RenderModels`-touching methods, through
accessor fields on `Common.hooks` installed at boot by
`mp_engine_server::hook_install` / `mp_renderer::hook_install`. Every
host-consuming C-track function takes `view: &mut EngineHostView` as its single
world parameter; `EngineHooks` fields and `CmdFunction` carry the view
(amending, not reverting, the 2026-07-12 receiver-order and hook-table
rulings). §F code keeps its generic `&mut impl EngineHost` signatures and now
receives the live view where only `MockHost` could stand before. Amends
engine-fork-discovery ruling 43 (crate home moved `mp_engine_core` →
`mp_engine_qcommon`; slots instead of plain `&mut` fields); the ruling-43
split constructor is `mp_engine_core::engine_host_view(&mut Engine)`.

Rejected: a raw-pointer `LiveHost` over `Engine` (user, 2026-07-11) — a host
method mutating `Common` through a raw pointer while a caller frame holds
`&mut Common` violates the `noalias` contract on `&mut` parameters; real
miscompile risk, not theoretical.

Slot-cast discipline is PER-SLOT: a cast copies the raw pointer out
(`slot.as_raw()`), so the view stays usable while the cast borrow lives; sound
iff nothing called meanwhile casts the SAME slot again (casts of different
slots may nest). Hook-target functions get exactly the hook field's signature;
callers already holding the real receiver use the `_body` variant so no second
cast of the same slot is created (`SV_ShutdownGameProgs` pattern). The
one-state-in-Common exception (`Common.stringed`) uses documented
take/put-back at its `SE_*` wrappers.

Plan + worker spec: `docs/plans/2026-07-11-host-seam-restructure.md`,
`docs/plans/2026-07-11-host-seam-worker-spec.md`.

## DEC-24 — Safe-state migration: staged, `Option<EntityId>` now, free fns forever (2026-07-12)

The `mp_game` safe-state migration runs as the six-stage plan in
`docs/plans/2026-07-12-safe-state-migration.md` (the authority; decisions
§3 there are user-settled). The three rulings most often at risk of
re-litigation:

- **Gameplay functions stay FREE FUNCTIONS taking `ctx` permanently** ("keep
  the free function") — no `impl GameContext`/`Game` method-ization of ported
  Raven gameplay fns, now or in roadmap Stage 2. Accessors on
  GameWorld/GameContext remain methods.
- **Entity nullability is `Option<EntityId>` from Stage 1 on** — no sentinel
  interim. Stored entity fields on `#[repr(C)]` seam structs keep raw layout
  (§D12); Option applies to params/locals/returns.
- **`GameContext` final shape** (realized in Stage 2):
  `{ world: &'a mut GameWorld, engine: &'a Engine }`, threaded as
  `ctx: &mut GameContext`, not `Copy`.

Rejected: method-ization (user, 2026-07-12); sentinel-first null migration
(user chose idiomatic-immediately); parallel shard execution for the
entangled hub files (serial mandatory there — every shard rewrites shared
dispatch tables and future shards' call sites; 2-wide worktree waves are
allowed only for the caller-disjoint tail, with serial human-reviewed
integration and the full referee before each commit).

## DEC-25 — `last-c-golden`: frozen pre-reshape reference point (2026-07-15)

The commit tagged `last-c-golden` (also branch `last-c-golden`, never
merged, never deleted) marks the last point where `mp_game` carries its
maximally C-faithful shape: the file-by-file translation audit is
complete (waves 1-6 + F1/F2/F3 sweeps, eight fix batches, every batch
adversarially validated and referee-byte-identical), and the very next
roadmap work (safe-state Stages 4-5, bg crate split, seam ratification —
DEC-24) deliberately reshapes internals away from the C structure per
porting-rules §A2.

What the mark certifies: audit-complete and referee-verified **on the
exercised tape set** (mock + real-map duel1). It does NOT certify proven
parity on referee-blind surface — the ratified verification tracks
(scenario burn-down, expression-width harness, dual-host engine referee;
see the wave-7 queue in `docs/audits/translation-audit-2026-07.md`) are
queued to strengthen the claim, and a second tag may be added when they
land. Use: permanent bisect/diff anchor if post-reshape referee runs
diverge — the tapes and oracle harness live in-tree, so the frozen point
remains independently checkable. CI is branch-filtered to master, so the
tag/branch cost no CI runs.

## DEC-26 — `gentity_t` homes in `mp_game`; the abi seam goes opaque (2026-07-15)

Raven defines `gentity_t` in `g_local.h` — game-private; the port's
placement in `mp_qshared` was a tiering workaround for the 22 `mp_abi`
syscall structs that name `*mut gentity_t`, and it forced the
`client`/`NPC`/`m_pVehicle` fields down to `*mut c_void` (~2,535 casts
across ~70 files). Ruled: move `gentity_t` to `mp_game` and restore the
real field types (`*mut gclient_t` / `*mut gNPC_t` / `*mut Vehicle_t`);
`mp_abi`/`mp_qshared` reference entities through an opaque
forward-declaration type (C's own `struct gentity_s;` idiom — the DEC-16
type-erased-slot pattern one tier up), with the cast pair confined to
trap packing at the seam. ABI-identical throughout (pointer-sized);
layout asserts and the `bgEntity_t` head contract (D12) move with the
struct. Executes as safe-state shard 4D.

## DEC-27 — Stage-5 bg split: full split, QAGAME residue severed first (2026-07-15)

The bg↔game boundary is already a trait wall (`GameCallbacks`, 28
methods, entity-num params only; `BgTraps` for engine services). Ruled:
all 11 bg files move to `mp_bg` (matching Raven's three-way-shared bg
set — game/cgame/ui per the vcproj listings), after the residual
in-place `#ifdef QAGAME` branches in `bg_misc`/`bg_pmove`/`bg_saber`/
`bg_slidemove` are lifted into `GameCallbacks` methods and `bg_saga`'s
`GameContext`/`GameWorld` params are severed through the same seam.
`GameBgTraps`/`GameCallbacksImpl` (the game-tier implementors) stay in
`mp_game`. The traits and `BgState`/`PmoveContext` move with the bodies.

## DEC-28 — `GameCallbacksImpl.world` raw store: sanctioned seam, not debt (2026-07-15)

The 13 construction sites storing `*mut GameWorld` into
`GameCallbacksImpl` are the honest expression of a two-sided seam:
`PmoveContext` must hold `&mut world.bg_state` while the callbacks
object re-enters full game logic that itself draws from `bg_state`
(RNG) — two live `&mut` into one world, unexpressible without the raw
store, and loan-out restructures break on exactly that reentrancy.
Ruled: sanctioned permanent, retagged `SEAM-BG-REENTRY` (replacing
`STAGE-2b: irreducible`); the STATE-D6 leaf-reborrow discipline inside
the method bodies is the containment. The remaining ~34 irreducible
markers stay transitional and are triaged during campaign 2c.

## DEC-29 — Campaign 2c outcome: accessor regime landed; residue classified (2026-07-16)

Six workflow waves on `safe-state-2c` converted the mp_game raw entity-deref
regime to point-of-use `GameWorld` accessor borrows: census 24,287 → 14,328
matched sites (41% eliminated, ~9,900 conversions, 81 one-file commits), the
six-scenario referee byte-identical after every wave (incl. duel1 combat over
the converted saber/force/weapon/combat paths), workspace + i686 + full test
suite green, and adversarial gate-2 audits over the referee-blind vehicle and
ICARUS/NPC-spawn surfaces returned zero divergences. Ruled classifications of
the remainder:

- **Permanent (allow-list):** trap-seam raw args (`entity_mut(id) as *mut`
  at the call, no deref), C-string/byteswap/variadic seams, Vehicle_t /
  vehicleInfo_t / bgEntity_t / playerState overlay derefs (§D12), the
  SEAM-BG-REENTRY store (DEC-28), and pool-client (`gClPtrs`) derefs read
  through the entity's own client pointer — `level.clients[i]` indexing is
  PROHIBITED unless the index is a proven real client slot (two wave-1
  defects; recipe rule 2b).
- **Deferred to the task-#7 regime design (not permanent):** gNPC_t
  (`NPCInfo`/`.NPC`) derefs (~no accessor exists), `bot_state_t` derefs,
  ambient-globals `&mut *NPCInfo` holds in pre-2d files, and the 112
  remaining STAGE-1 fn-top re-derives across 15 files — dominated by
  g_mover's multi-entity pusher machinery (50), w_saber's giant combat-trace
  fns (18 + whole-body G_Damage/player_die/ForceThrow tails), and g_client
  lifecycle fns. Greppable: `let \w+: \*mut gentity_t = ctx.entity_mut(`.
- **§19 divergences accepted in-campaign:** accessor bounds-panics where
  Raven had UB (g_vehicles G_AttachToVehicle / pilot-index, Howler_Combat
  enemy unwrap); each noted at the site.

`#![deny(unsafe_code)]` holds where files went fully clean (prelude,
g_arenas); broad deny-lint completeness waits on the deferred classes above.

## DEC-30 — Bot AI stays oracle-faithful; OpenJK bot improvements rejected (2026-07-16)

Live five-lane A/B (2026-07-16, mp/ffa1, 8 bots) showed bot combat quality
splits cleanly by game module, not engine: our module and the compiled
oracle dylib produce identically weak bots (poor aim, weak target
commitment) on every engine, while OpenJK's basejka module produces
noticeably better bots on every engine. Our port is faithful — the referee's
byte-identical duel1 verdict and targeted spot-checks (.jkb personality
parsing incl. CRLF handling, ScanForEnemies, BotAimOffsetGoalAngles,
BotChangeViewAngles) all confirm parity with the oracle source.

**Ruling (user, 2026-07-16): keep bot behavior oracle-faithful. Do not
adopt OpenJK's bot-AI changes.** The observed "bad" bot feel is retail-SDK
behavior, not a defect; no bot-quality hunt should be reopened against the
oracle baseline. Reference if ever revisited: local OpenJK checkout at
`~/Developer/Milo/OpenJK` (game/ai_*.c, g_bot.c); note OpenJK server lanes
also differ environmentally (sv_fps 40, g_maxForceRank 7).

## DEC-31 — Safe-state mechanical stages frozen; idiom era opens (2026-07-16)

**Ruling (user, 2026-07-16): the remaining mechanical safe-state work
(STAGE-1 tails, Stage-2 world-borrow flip as a standalone pass) is
abandoned — superseded by per-subsystem idiomatic rewrites ("the idiom
era").** The goal shifts from incrementally hardening the transcribed C
shape to producing Rust-centric end results: each slice rewrites one
subsystem into idiomatic Rust, gated by byte-identical referee replay,
and the finished mp_game becomes the exemplar for porting cgame/ui
directly into idiomatic shape (cgame/ui deferred until then).

Settled parameters of the era:

- **Slice one: g_items.** Chosen as first exemplar (bounded, referee-hot,
  forces the item-table/bg call surface).
- **bg idiomizes call-surface-by-call-surface.** Each game slice
  idiomizes the bg APIs it actually consumes (g_items pulls
  `gitem_t`→`GItem`/`ItemId` and the `BG_FindItem*` signatures);
  bg_pmove internals wait for a movement slice. No throwaway adapters,
  no dedicated bg slice.
- **Naming: Raven names stay** (`BG_FindItemForWeapon`, `RegisterItem`,
  …) — grep-parity with the oracle is load-bearing for referee debugging
  and audits. Only genuinely new constructs with no Raven counterpart
  (`ItemId`, `ItemKind`, methods on them) get Rust-style names.
- **Referee tapes lead the slices.** A bots-heavy mp/ffa1 tape is
  recorded before slice one; every referee-blind subsystem (NPC,
  vehicle, mover) gets a tape before its slice starts.
- **ABI seam unchanged**: engine-crossing types stay `#[repr(C)]` with
  layout asserts; wire-visible values must stay byte-identical.
- **Two-`&mut` rule**: split-borrow (`entity_pair_mut`) only at
  structurally disjoint sites (e.g. pusher loops excluding self);
  `G_Damage` stays on sequential re-acquire — suicide aliases
  target/attacker/inflictor (`g_cmds.c:1193`).
- **Carried forward for their future slices** (from the task-#7
  sit-down): gNPC_t → owned slab + accessor; pool clients stay DEC-29
  raw until their slice; bot_state_t gets full conversion. The
  `&mut *NPCInfo` UB carve-out landed ahead of the era (4bd836a4); the
  remaining held gentity_t/gclient_t derefs retire slice-by-slice.
- Function-static atomics folded to owned state (5ed6e1b0); B3
  exception list is empty.

## DEC-32 — Dedup campaign ratified: one home per duplicated implementation (2026-07-18)

The whole-workspace duplicate sweep (49 reader agents over every file in
`crates/`; merged inventory at `docs/audits/duplicate-inventory-2026-07-18.md`)
found 6 behavioral-divergence clusters, ~48 same-side centralization clusters,
16 byte-identical SP/MP twins, and 7 test-support clusters. The campaign plan
(`docs/plans/2026-07-18-dedup-campaign.md`) is ratified with four user-settled
choices:

1. **`crates/native/string` is created** as the C-string runtime home
   (`c_str*` family, `c_atoi`, ptr→String, `Com_Filter` glob family,
   `VALIDSTRING`, GP2 tokenizer); `native_platform` re-exports from it.
2. **`c_atoi` standardizes on strtol-style clamp overflow semantics**
   (retail win32 msvcrt behavior); the three divergent copies converge.
3. **All 14 byte-identical SP/MP twin clusters hoist into `native`**
   (per-side re-exports, layout asserts retained). Deliberately-divergent
   ABI twins are untouched.
4. **The `QSharedScratch`-threaded string shape is canonical** —
   `mp_qshared`'s `static mut` `q_string`/`COM_Parse` copies retire in favor
   of the threaded impls (closes the §B3 violation inside qshared).

Oracle-inherited duplication (inventory category 5) stays faithful per
porting-rules §A2/§20 — no action pre-parity.

## DEC-33 — l_struct + iteminfo/weaponinfo strings parked permanently (2026-07-23)

`weaponinfo_t` is memcpy'd whole across the game ABI (`BotGetWeaponInfo`,
`sv_game.rs`), freezing its `[c_char; MAX_STRINGFIELD]` fields and the
`l_struct` fielddef offset filler that populates it. Converting the goal-only
`iteminfo_t` strings would fork a duplicate parser beside l_struct for no wire
or safety benefit. Ruling: the l_struct machinery, `iteminfo_t`/
`weaponinfo_t`/`projectileinfo_t` string fields, and the `BotGoalName` export's
interior stay as-is permanently. Internal-quality-only residue; cite this
ruling rather than re-opening.

## DEC-34 — bg_lib qsort body canonical; msvcrt tie-order question closed (2026-07-23)

Retail's `JK2_game.vcproj` excludes `bg_lib.c` from the native game DLL, so the
retail win32 DLL bound msvcrt's qsort, whose tie permutation differs from the
BSD Bentley-McIlroy body `native_sort` transcribes. Ruling: `native_sort`'s
bg_lib body is canonical everywhere; msvcrt's tie order is never reproduced.
Grounds:

1. The referee oracle binds Raven's own body, not msvcrt: `bg_lib.c`'s qsort
   sits outside every `Q3_VM` guard and shadows libc in the oracle dylib
   (`_qsort` defined `T`, no undefined import — `g_main.rs` design note,
   2026-07-06). A/B tie parity is exact by construction, including the
   first-frame all-scores-equal tie.
2. There is no single retail tie order: win32 bound msvcrt, but Mac retail and
   Linux dedicated bound their platforms' libcs — tie order was already
   platform-variant across the shipped ecosystem.
3. No tie is compat-observable at any call site: `SortRanks`/`SortClients` tie
   order is scoreboard presentation only; `paksort` and
   `SV_QsortEntityNumbers` cannot see equal keys; ghoul2 `QsortDistance` is an
   inconsistent comparator already ruled onto `total_cmp` (§19); `vm_fns`
   profiling is dev-only output.
4. Microsoft's CRT source is proprietary — not transcribable under the
   oracle-fidelity method even if wanted.

## DEC-35 — Ghoul2 block ownership: views at the seam, parsed-once sidecar (2026-07-23)

Sit-down ruling for task #17 (full plan:
`docs/plans/2026-07-23-ghoul2-ownership.md`). The `mdx/` view module hoists
from `mp_engine_ghoul2` to `mp_host_interface`; `EngineHost::model_mdxm/mdxa`
return `MdxmView`/`MdxaView` instead of `*mut c_void`; `CGhoul2Info` and
`CBoneCache` store views (one conjure site at the EngineHost seam, down from
59), with `G2_SetupModelPointers`-style revalidation extended to `CBoneCache`;
the renderer builds an owned `MdxaParsed`/`MdxmParsed` index once at ingest
(header constants, skel table, surface hierarchy — data read more than once
per model lifetime), handed out as a Copy `MdxaRef`/`MdxmRef` pair; per-frame
data (compressed bone pool, verts) stays view-based. Amends the letter of
G2SV-D5/D15 (which forced the `*mut c_void` seam) while keeping their
substance: no ghoul2→renderer crate edge, no duplicate file-parse path.

## DEC-36 — ui root types: UiWorld/MenuSystem/DisplayContext (U2 sit-down, 2026-07-24)

Ratifies the ui-port root-type set (plan:
`docs/plans/2026-07-24-client-port/ui-plan.md`, stage U2; rulings D1-D8):

1. **`UiWorld`** — the owned ui spine: `uiInfo_t` + ui_force.c globals +
   every file-scope static folded into fields; `String`/`bool`/`Vec`
   throughout (the frozen-vs-free census proves all of it Class C /
   module-private); Raven field names kept.
2. **`MenuSystem`** — ui_shared.c's menu framework owned by composition
   (`UiWorld.menus`): menuDef/itemDef arena + index handles replacing the
   raw-pointer graph; `String_Alloc` intern pool → owned string table;
   open-menu stack as indices.
3. **`DisplayContext`** — an idiomatic trait REPLACING Raven's
   `displayContextDef_t` fn-pointer struct; the faithful repr(C) port and
   its layout asserts retire (blast radius verified: one MP consumer,
   `uiInfo_t.uiDC`; SP's differently-shaped sibling is unrelated). cgame
   later implements the same trait for `cgDC`.
4. **`UiContext`** — `{ world: &mut UiWorld, engine }`, owned by the vmMain
   entrypoints and threaded inward; analog of `GameContext`.
5. **bg per-module arms** — Raven's `#ifdef WE_ARE_IN_THE_UI`/`UI_EXPORTS`/
   `CGAME` branches in bg_vehicleLoad/bg_misc/bg_saga/bg_g2_utils become
   `GameCallbacks` trait dispatch: the trait gains the methods the non-game
   arms need (e.g. shader registration; game's impl stays faithful to its
   arm, including Raven's commented-out no-ops); ui implements the trait
   over its trap layer. ui reuses mp_bg's animation module instead of
   porting Raven's hand-synced `UI_ParseAnimationFile` fork.
6. **ABI arm** — legacy `vmMain`+`dllEntry` (the jampgame precedent; loads
   under OpenJK's fallback arm and retail), basename `ui<ARCH><EXT>`.
7. **Dead surface (§20)** — `ui_players.c` and `ui_util.c` are not compiled
   into MP ui (absent from ui.vcproj and ui.q3asm; OpenJK deleted the files
   outright); dropped, along with ui_main.c's `UI_DrawOpponent`/
   `UI_DrawPlayerModel` static family (only call sites commented out).
8. **Seam cleanups** — the unused duplicate `uiExport_t`/`uiImport_t` enums
   in `mp_abi::ui::public` are deleted; copy-pasted `Cg*` wrapper type names
   inside `mp_abi::ui::syscalls` renamed `Ui*`.

**U3 addenda (2026-07-24):** (9) `DisplayContext` carries ONLY the callback
surface — `MenuSystem`/`DisplayState` thread beside it as struct fields of the
host context (the `GameContext` field-split-borrow precedent), never through
trait accessors. (10) The force-mastery anonymous enum hoisted to its DEC-32
canonical home `mp_bg::public::force_mastery` (and `MAX_FORCE_RANK` to
`mp_qshared::shared::force_powers`); the `mp_game`/`mp_ui` private copies
deleted.

**U4 addenda (2026-07-24):** (11) **ui owns a `BgState`** —
`UiWorld.bg_state: BgState`, mirroring Raven's ui link unit compiling the bg
files itself (`WE_ARE_IN_THE_UI`): its own rand state (`bg_state.rng` — the
saber-flicker `crandom()`/`Q_irand` route), its own siege tables, its own
parse scratch, and its own `BG_Alloc` pool at Raven's `UI_EXPORT` arm
(`MAX_POOL_SIZE_UI = 512000`, `bg_misc.c:3311-3316`; `BgState` gained
`with_pool_size`, §F20 duplicate-don't-unify — closes the parked pool-size
question; cgame's 2048000 arm noted for later). `ui_saber.rs` twins thread
`bg: &mut BgState` like their `bg_saberLoad.rs` counterparts. (12) Ruling 9
fallout, host side: `mp_ui` fns that call `DC->` callbacks take
`dc: &mut dyn DisplayContext` as a parameter beside their state params (first
site `UI_Version`); the concrete implementor is a U5-built carrier over
split borrows of `UiWorld` — `UiContext` itself can NOT implement the trait
(it owns `world.menus`/`world.uiDC`, which must stay independently borrowable
while `dc` is live).

## DEC-37 — renderer deviation charter + architecture (R0 sit-down, 2026-07-25)

Ratifies the R0 deviation charter and the renderer architecture, all 17 agenda
items ruled (brief prepared against oracle + external references; the brief
itself stays out of repo docs per the external-reference rule):

1. **Deviation charter, verbatim.** The renderer interior is free — no oracle
   matching. Three edges stay fixed: the VM trap seam (`CG_R_*`/`UI_R_*`
   syscalls + the ported `refEntity_t`/`refdef_t` family), asset semantics
   (shader-script grammar, BSP/model/font/image/roq meaning, retail look), and
   the sim-visible model/collision subset. Presentation-side threading is in
   scope on mainline. (There is no coherent Raven interior to transcribe: MP
   gutted Q3's SMP — empty `R_InitCommandBuffers`, backend inline,
   `oracle/codemp/renderer/tr_cmds.cpp:72-80,105-107`.)
2. **Threading topology** — two long-lived threads + shared rayon pool: sim/VM
   thread builds an owned `FrameData` (sealed at EndFrame, posted over a
   bounded channel with recycled buffers); render thread owns the GPU and does
   cull → sort → skinning dispatch → encode → submit → present; pool does
   asset decode + skinning/deform jobs.
3. **State-partition law** — `RenderAssets` (CPU, immutable-after-publish,
   Arc-shared, readable from the sim thread) vs `GpuResources`
   (render-thread-only). Invariant, verified exhaustively at R2: **no trap
   query may touch GPU state.** Every synchronous seam query reads CPU data
   only.
4. **`refexport_t` deleted.** MP statically links the renderer (no
   `refimport_t`; eight cgame traps bypass the table —
   `oracle/codemp/client/cl_cgame.cpp:943-1720`); nothing crosses a C ABI
   through it in our stack, so the repr(C) port retires at R2. The boundary is
   a plain Rust trait / direct calls.
5. **Shader translation: one renderer, swappable shader backend** behind a
   stage-program seam over the parsed `ShaderDef` IR. Backend #1 (baseline,
   built first): faithful uber-shader — one WGSL program, per-stage uniform
   records driving the closed `CGEN_*`/`AGEN_*`/`TCGEN_*`/`TMOD_*` grammar;
   the parity reference and the ui-slice dependency. Backend #2 (early
   second): **PBR uber-shader** consuming AI-generated material sidecars
   (normal/roughness/AO) — sidecar generation is a **separate tool**
   (disk-cached, content-hash keyed; ML runtime never enters the game
   binary); lighting directionality (lightgrid-derived, later deluxe
   maps/cubemap probes) scoped to this backend. Per-shader WGSL codegen stays
   an optional future backend-internal optimization behind the same seam.
   Frame-level AI (upscaling/post) is orthogonal — it lives in the pass
   graph, not the shader backends.
6. **Pipelines** — lazy `HashMap<PipelineKey, RenderPipeline>` keyed on real
   fixed-function state only (blend, depth compare/write, cull, polygon
   offset, color mask, vertex layout, target format); `alphaFunc` is a
   fragment `discard`, not a key; `EndRegistration` warms the cache.
7. **Geometry** — static world vertex/index buffers built once at map load
   (grouped by shader+lightmap), curves tessellated once, md3 keyframes
   uploaded once with VS lerp; dynamic ring buffer only for 2D/polys/sprites/
   marks/deformed surfaces. Dlights become a per-draw light list evaluated in
   the fragment shader (`MAX_DLIGHTS 32` bound), replacing the extra tess
   pass. Raven's per-frame 1000-vertex `tess` rebuild is discarded.
8. **Deforms split** — `wave`/`normal`/`bulge`/`move` in the vertex shader;
   `autosprite`/`autosprite2`/`text0-7` as CPU pool jobs into the ring buffer.
9. **Ghoul2: GPU skinning** — the already-ported bone math
   (`crates/mp/engine/ghoul2/src/render/`) produces the matrix palette (one
   bone path shared with sim), uploaded to a storage buffer; the VS blends
   from the packed mdxm weights (two parallel attribute streams → two vertex
   buffers). CPU-pool skinning retained as fallback + golden reference.
10. **Gamma/overbright: numbers not mechanisms** — force `overbrightBits = 1`
    unconditionally (windowed = fullscreen retail); keep `identityLight`, the
    lightmap/lightgrid shift with renormalize-don't-clamp
    (`oracle/codemp/renderer/tr_bsp.cpp` behavior) exactly, the fog-color
    premultiply, the cinematic tint, and the 2×2 box mip filter; gamma +
    overbright applied as an output pass using retail's exact
    `pow(x, 1/gamma) << shift` curve; only `r_intensity` bakes into textures.
    Hardware ramps and the windowed-mode overbright clamp are discarded.
11. **Asset cache** — generation-counted handle tables over
    `Arc<RenderAssets>`; registration stays synchronous (retail contract; no
    provisional handles) with decode/upload parallelized inside the call.
    Retail identity quirks reproduced: lower-cased extension-stripped image
    cache key, mismatched-params cache hit only warns, same name + different
    lightmap config = distinct shader instance (no `_2d`/`_vertex` name
    mangling). Material sidecars from ruling 5 are read here.
12. **Flares + the standing tiebreak** — implement the oracle's deferred
    flare behavior (`oracle/codemp/renderer/tr_flares.cpp`); wherever retail
    and OpenJK diverge, **the oracle wins** — retail is the asset-semantics
    edge.
13. **Scope fences** — MDR dropped (no loader references it); Xbox/
    `VV_LIGHTING`/`bumpmap`/`specularmap` dropped (compiled out of shipped
    PC build); `RE_Scissor` does not exist, not invented; mini-refentity
    chain ports the live pad-and-forward shim only (real chain is `#if 0`);
    CJK glyph pages deferred (`Language_*` stays in the seam). Terrain/RMG,
    weather/world effects, automap, surface sprites, dynamic glow,
    distortion/refraction all stay in scope, sequenced last.
14. **Slice order: 2D-first** — R4a 2D backend (window/wgpu/StretchPic/fonts/
    shader parse/asset tables — runs the entire ui module) → R4b
    `RDF_NOWORLDMODEL` scenes (completes ui; GPU skinning) → R3 world
    (oracle-golden-gated; PBR backend starts after) → R4c effects tail → R5
    dev harness + cl_* island.
15. **Validation** — layered: oracle-differential CPU goldens (§18 pattern,
    `tools/renderer-oracle/`, no GPU in CI) + draw-list goldens
    (self-referential) + perceptual image comparison at fixed camera poses +
    a shader-zoo scene. CPU/draw-list goldens gate CI; the image gate is a
    manual wave-boundary step.
16. **Crate layout** — `crates/mp/renderer` stays CPU-only (assets, parse,
    cull/sort, existing `tr_model`; `jampded`'s link is untouched, goldens
    run GPU-free); a sibling GPU crate owns the wgpu backend; `native/gpu` +
    winit in `native/platform` per DEC-01/DEC-02.
17. **SP: design both modes now** (user ruling, against the brief's
    MP-first recommendation): R2's seam inventory covers SP's `refEntity_t`/
    renderfx variants and its statically-linked call sites alongside MP's
    traps; interior scene/asset types are mode-agnostic by construction; SP
    divergences become edge adapters + quirk flags, never a second renderer.

**DEC-37 R2 addenda (ratified 2026-07-25, one-by-one sit-down):**

- **A1. `FrameData` = lean design + disposition table.** The channel message
  (ruling 2) is designed from the frozen per-frame trap surface (the 57
  `CG_R_*`/`UI_R_*` `Args` types + refexport call list), specified as an
  **ordered event stream** (2D commands and scene renders keep their
  interleaving), NOT a field mirror of `backEndData_t` — which mixes frontend
  inputs with `drawSurfs`, a frontend *output* that never crosses our channel
  (cull/sort run render-side per ruling 2). R2 must produce a **disposition
  table**: every `backEndData_t` field and every render command → crosses in
  `FrameData` / stays render-side / dead.
- **A2. Registries: four typed arenas.** image/shader/skin/model each get
  their own arena (matching the oracle's four independent `MAX_*` arrays)
  with `(index: u32, generation: u32)` handles — filling in ruling 11's
  committed generation-counted shape; per-kind typing catches cross-kind
  handle mixups at compile time.
- **A3. Compose seam types by value.** `trRefEntity_t`/`trRefdef_t` wrap
  `mp_qshared`'s `refEntity_t`/`refdef_t` by value at R2 rewrite time
  (matching the oracle's own `refEntity_t e;` composition), collapsing the
  duplicated field sets so the layout-assert block lands once for
  ui/cgame/renderer.
  *(Amended at the R2 Gate-2 round: `trRefdef_t` composition is
  layout-impossible — it is NOT a superset of `refdef_t` (field-level proof
  in `docs/subsystems/renderer-r2-design.md` R2-D6); A3 applies to
  `trRefEntity_t` alone, which is already landed.)*
- **A4. Housekeeping approved:** read `tr_image.cpp`'s image-table
  declaration before finalizing the RenderAssets registry count (the flare
  half shrank: R1 established `tr_flares.cpp` is dead in retail MP — absent
  from `jk2mp.vcproj`, sole caller commented out at `tr_backend.cpp:1244` —
  so ruling 12's flare scope resolves to *no MP flares*); spot-check
  `mnode_s.rs` against the non-XBOX `tr_local.h:917-934` branch before R2
  closes; DEC-37 bucket assignments are provisional until R3 waves exercise
  real call sites. `tr_quicksprite` is NOT SP-only (222-line
  `tr_quicksprite.cpp` is in the retail MP compile set and the R1 srcglob;
  simply un-ported R3 scope) — ruling 17's mode-agnostic-interior claim
  stands without asterisk.

**DEC-37 R2 addenda, second sit-down (ratified 2026-07-26):**

- **A5. Arena capacity semantics.** Shader/skin/model arenas soft-cap at
  their oracle `MAX_*` constants (16384/1024/1024) and reproduce retail's
  table-full console warning; registration failure returns the oracle's
  failure value (default/0 handle), not a Rust `Result`. The image arena is
  unbounded, matching its real oracle backing store (`tr_image.cpp`'s
  `AllocatedImages` std::map; the `MAX_DRAWIMAGES` check is commented out in
  retail) — with a name→handle map beside it mirroring the lower-cased
  extension-stripped key scheme.
- **A6. Light styles are synchronous CPU state**, not `FrameData` events:
  `R_Set/GetLightStyle` mutate/read a live RenderAssets-adjacent table at
  trap time; frames snapshot the table at scene-render marks. R3 caveat: the
  wave porting the backend style consumer must verify snapshot-vs-live
  timing against the oracle.
- **A7. Defers approved**: FrameData buffer-recycling mechanics settle at R4
  (lean: fixed 2-3 buffer pool + return channel; R2 freezes only the
  event-stream shape); `RC_AUTO_MAP`'s full command shape gets its targeted
  oracle read at the first automap wave; `subImageCommand_t` dead-vs-internal
  gets its grep before R3 scope-freezes — **CLOSED 2026-07-27: dead.** The
  grep found `subImageCommand_t` defined once (`tr_local.h:2201`) with zero
  references in any oracle `.cpp` (no issuer, no `RC_` id, no backend
  dispatch arm) and no `RE_SubImage` anywhere; the type is already ported
  with layout asserts (`tr_local/sub_image_command_t.rs`), nothing further
  owed; the generic `Handle<K>`/`Arena<T>`
  infra carries a doc-comment citing the `AlignedBytes` justified-exception
  precedent (no `Source:` line — new Rust infra implementing ruling 11).
- **A8. The R2 design doc is RATIFIED** and lands as
  `docs/subsystems/renderer-r2-design.md` (owned-world sketch, A1
  disposition table, seam-composition plan, SP/MP edge adapters,
  verification strategy, slice hooks). R2 is closed; R3 tooling is
  unblocked.

**DEC-37 R2 addenda, doc-review fix round (ratified 2026-07-26):**

- **A5 amendment — per-registry failure semantics (measured).** Retail's
  overflow behavior differs per registry: shaders warn and return
  `tr.defaultShader` (`oracle/codemp/renderer/tr_shader.cpp:2760-2761`),
  skins warn and return handle 0 (`tr_image.cpp:3139-3141`), models are
  SILENT and return 0 (`tr_model.cpp:614-616`, `:1044-1045`). Fallback
  VALUES are reproduced exactly (rendering-observable — keeps the
  differential rigs free of phantom diffs); warnings keep retail's
  shader/skin prints PLUS a port-added, clearly-marked warning on the
  silent model overflow (charter interior freedom, debugging aid).
  `Arena<T>` gains a per-registry fallback handle so `tr.defaultShader` is
  expressible. *(Fallback-field mechanism superseded by A12's slot-0
  pre-population — the default entry lives IN slot 0; failure returns
  `Handle{0,0}`.)*
- **A9. RenderAssets mutation path = sim-owned master + republish.** The
  sim/registration side owns the master `RenderAssets`; synchronous
  mutations (register shader/model/skin, remap_shader) go through
  `Arc::make_mut` copy-on-write and the new `Arc` publishes at the next
  frame boundary. No locks; the render thread's view is immutable within a
  frame. Light styles stay A6-adjacent (a separate sim-owned table
  snapshotted at scene marks), not inside the `Arc`.

**DEC-37 R2 addenda, Gate-2 re-review round (ratified 2026-07-26):**

- **A10. Automap init is a sim-side A9 mutation.** `CG_R_INITWIREFRAMEAUTO`'s
  live arm (`oracle/codemp/renderer/tr_world.cpp:1205-1231`) rebuilds the
  wireframe automap by walking the world nodes and returns validity
  synchronously — an event cannot answer the caller, so the wireframe data
  becomes a `RenderAssets` member built sim-side through the A9 mutation
  path (pure CPU work; ruling 3 intact). The doc's prior
  synchronous-read classification is corrected.
- **A11. Light-style snapshot carrier.** `FrameEvent::RenderScene` gains
  `light_styles: [[u8; 4]; MAX_LIGHT_STYLES]` (256 bytes) — the operational
  form of A6's snapshot-at-scene-marks; the render-side consumers
  (`tr_surface`/`tr_shade`/`tr_light`) read the frame's snapshot. R3
  snapshot-timing caveat unchanged.
- **A12. Slot-0 pre-population.** Each capped arena pre-populates slot 0 at
  init with the oracle's reserved default entry (models[0] MOD_BAD,
  skins[0] default skin, shader 0 default shader);
  `Handle{index:0, generation:0}` IS that live default and every oracle
  `return 0` failure path returns it — `qhandle_t` 0 maps to slot 0 as the
  identity at the seam, so retail's render-the-default-on-failure behavior
  falls out exactly.

**Addenda A13 (2026-07-26, R3-prep state-home sit-down):** the R2 doc froze
the root types but homed only the globals its rows name; the packet
generator's UNMAPPED report (336 globals) was ruled as five families,
one-by-one:

- **A13.1 Renderer cvars (125 `cvar_t*` handles).** One owned
  `RendererCvars` struct, one field per cvar, registered in `R_Register`
  and threaded as a carrier — the engine-island `EngineCvars` precedent.
  Reads go through the engine cvar table live; R4 may later snapshot what
  the render thread needs (Q3-SMP-style FrameData snapshot REJECTED for
  now as premature).
- **A13.2 GL/WGL entry pointers (52 `qgl*`/`qwgl*`).** No R3 home — per
  DEC-01 they dissolve into the R4 wgpu rewrite. Backend fns' GL calls
  carry `// DEFERRED: R4` cites; a frontend fn must never grow a GL
  dependency. Stub-GL-trait REJECTED (throwaway surface against DEC-01's
  not-a-GL-transcription ruling).
- **A13.3 TU-local working statics (~120 across 20 files).** Blanket rule:
  each file's statics become an owned per-subsystem state struct (§B3/B6),
  placed sim-side or render-side per where the subsystem executes, **named
  by the wave that ports the file**. Genuinely-const tables → `const`;
  init-once tables (e.g. `s_gammatable`) → fields filled at subsystem
  init. A static that crosses the sim/render boundary escalates instead of
  being placed silently.
- **A13.4 Shader-parse statics (16).** Parse scratch
  (`shader`/`stages`/`texMods`/`collapse`) → a `ShaderParseState` local
  threaded through the `ParseShader` chain, alive only during one parse —
  NOT stored state. The shader-text cache
  (`s_shaderText`/`shaderTextHashTable`/`deferLoad`) → `RenderAssets`
  beside the ruled `shader_lookup`.
- **A13.5 Ghoul2 profiling timers (19 `G2PerformanceTimer_*`/`G2Time_*`).**
  Dropped per §20 dead surface with a module-doc note:
  `G2_PERFORMANCE_ANALYSIS` is defined only `#ifndef FINAL_BUILD`
  (`oracle/codemp/game/q_shared.h:44-46`) — retail compiles the timers and
  their call sites out.

## DEC-38 — ui U5 rulings: DisplayContext carrier, WORLD lifetime, siege gating (2026-07-25)

Three user rulings from the U5 one-by-one sit-down:

1. **DisplayContext carrier = thread-the-borrow.** The 7 re-entrant trait
   methods (`ownerDrawItem`, `runScript`, `deferScript`, `ownerDrawHandleKey`,
   `feederCount`, `feederItemImage`, `feederSelection`) are widened to take
   the CALLER's borrows (`menus: &mut MenuSystem`, + `ds` where needed) —
   mutations are visible on return, no aliasing (porting-rule B4 "state is
   threaded, not reached"). Consequence: `MenuSystem` hoists out of the
   world-struct borrow that the carrier holds — the carrier is built over
   split borrows of `UiWorld` (Engine + the non-menus remainder), and every
   `ctx.world.menus` path updates mechanically. Raw-reborrow (`*mut UiWorld`
   in the carrier while callers hold `&mut MenuSystem`) was REJECTED: two
   live mutable paths to the same `MenuSystem` is UB in Rust's aliasing
   model, not confined unsafe. Routing facts (session survey): 47/74 methods
   are pure trap forwarders (14 need trivial new `trap.rs` wrappers), 27
   route to ported `UI_*` logic, 59 have live `dc.` call sites; the 15
   zero-call methods are all trap forwarders (spot-check vs §20 dead surface
   before implementing them). `feederItemText`'s dc-free signature is
   faithful to Raven — do not "fix" it.

   **Revision (same day, ratified):** implementation proved the carrier-type
   wording unbuildable — a carrier holding the world (safe or raw) beside the
   ported fns' live `&mut UiContext` is the same two-mutable-paths hazard this
   ruling rejects (rustc E0499 proof; session scratchpad borrowtest/). Ruled
   shape: **`UiContext { world, engine }` implements `DisplayContext` itself**
   — the hoist of `menus`/`uiDC` out of `UiWorld` dissolves DEC-36 addendum
   12's objection — and `mp_ui` fns take `(ctx, menus, ds)` in place of
   `(ctx, dc)`, passing `ctx` where a dc is wanted. Zero unsafe, `ds` stays
   `&DisplayState` (measured: no dc-routed target writes `DisplayState`), all
   Kind-B trait methods widen to take the caller's `menus`/`ds` (not just the
   7), and `UI_StopCinematic`/`UI_PlayCinematic` shed `ctx` for
   `(world, engine)`. Of the 15 dead-call trait methods, 7 are §20 dead
   surface (no `DC->` call site in the oracle tree) and panic-with-subject;
   the other 8 back live untranscribed `trap_*` sites and got wrappers.
2. **ui WORLD lifetime → STATE-D6 addendum** (recorded in
   `docs/architecture/state-ownership.md` same-day): per-module world
   lifetime follows the module's own Raven semantics — construct-on-INIT for
   game (`level` re-created per map), lazy-and-persistent for ui (`uiInfo`
   file-scope static, never reinitialized, nothing freed on shutdown).
3. **Siege class-determination gating**: `GameCallbacks::siege_class_shader`
   widens to `-> (c_int /* handle */, bool /* run class determination */)`.
   game's impl returns `(0, true)` always (QAGAME runs the block
   unconditionally, `bg_saga.c:994-1039` `#ifdef` arm); ui's returns
   `(handle, handle != 0)` (the `#else` arm's `else` gating). `bg_saga.rs`'s
   class-determination block branches on the bool. Closes the divergence
   activated by the ui implementors (87abf20a review finding 12; task #36).

## DEC-39 — post-parity comment sweep: Raven-verbatim + cites, two-pass shape (2026-07-26)

User ruling. Timing: **post-parity in all modules** (after jampgame + engine +
ui + cgame + renderer are done) — not before, because in-flight waves keep
producing comments in the current house style and the port tooling/review loop
still consumes the doc comments until then.

1. **Pass 1 — strip to Raven-verbatim + cites.** Deletion-heavy sweep of every
   crate: port-added explanatory prose is removed except where the code is
   genuinely hard to follow (short, load-bearing notes only); PORT-NOTE
   scaffolds deleted (per the standing comment policy). What stays, exactly:
   - **Raven comments, verbatim** — including the entity `QUAKED`/spawnflag
     documentation blocks with their original Raven formatting preserved
     character-for-character (user: "the raven formatting for the entities
     where we see what the spawnflags mean").
   - **`Source:` cites** — retained; they are load-bearing infrastructure
     (badge/assert tooling greps them, oracle review navigates by them, the
     SP-as-diff port depends on them). May be compressed to a bare
     `// codemp/...:<line>` form.
   - Safety-invariant comments on `unsafe`, and the layout-assert blocks.
2. **Pass 2 — optional organic-style rewrite** (the "reads like a real team
   wrote it" pass): style details deferred to a sit-down when pass 1 is
   reached — not designed now (user: "when it comes to it, we'll figure that
   out").
3. **Mechanical gate for both passes:** comments-only — strip comments from
   both sides of every file diff and require byte-identical token streams; any
   worker that touched code is auto-rejected. `cargo build --workspace` green;
   referee run as belt-and-suspenders though comments cannot affect it.


## DEC-40 — client-track builds take the non-DEDICATED leg (R3 waves 7-13 close-out, 2026-07-27)

**Ruling (user-ratified 2026-07-27):** every client-track transcription — the
R3 renderer remainder and the coming cgame/`cl_*` waves — takes the
**non-DEDICATED** (`#ifndef DEDICATED`) leg of Raven's compile-time split.
Raven compiles the same TUs into both jamp.exe (DEDICATED undefined) and
jampDed.exe (defined); our dedicated server never links a renderer beyond its
headless subset, so all new renderer/client logic is jamp.exe-based.

1. **The jampded headless subset keeps its dedicated-arm dispositions** —
   `ServerLoadMDXA`/`ServerLoadMDXM`, `RE_RegisterServerModel`, the server
   skins path (`gServerSkinHack`/`R_FindServerShader`), and every drop the
   TRM-D2…D5 rulings scoped to the server link set. Those rulings are
   *scoped*, not overruled: they never license dropping client-leg code in
   client-track files.
2. **Runtime switches are not the compile flag.** Where Raven splits at
   runtime (`com_dedicated` cvar, `com_cl_running`), both arms are
   transcribed and the check decides at runtime (existing precedent:
   `R_FindImageFile`, `R_InitShaders(server: bool)`).
3. **A client-leg call that cannot be wired yet** gets a greppable
   `//TODO: Port <subject>` naming this DEC and the real gap — never a
   silent drop citing a jampded precedent (the waves 7-13 defect this DEC
   closes: two porters dropped `RE_LoadWorldMap_Actual`-on-`#`-models and
   `R_SyncRenderThread` as "DEDICATED is live").
4. **Packet preambles state the policy** — R3-tail planning defect #4/#5:
   wave packets for client-track TUs must declare "client track: take the
   `#ifndef DEDICATED` leg, cite DEC-40" so porters stop re-deriving it.

Applied in the waves 7-13 fix round (commit 255ec091); review survey: 17 of
19 guarded sites in that diff already took the client leg.

## DEC-41 — `M_PI` expands as the math.h f64 (user-ratified 2026-07-27)

**Ruling:** wherever Raven's `M_PI` macro appears, the port treats it as the
**math.h double** and keeps C's double-promotion trajectory for that
expression, rounding to f32 exactly where C rounds (ruling-12 discipline).

Background: `q_shared.h:547-549` guards a *float*-suffixed fallback
(`3.14159265358979323846f`) behind `#ifndef M_PI`; MSVC's math.h defines
nothing (no `_USE_MATH_DEFINES` anywhere in the oracle), so retail Windows
binaries computed with the f32 literal, while clang/glibc builds — every
differential oracle we can build, and OpenJK on macOS/Linux (OpenJK keeps
the identical guard, `shared/qcommon/q_math.h:40`) — get the double.

1. **Scope is the literal's width only.** Storage stays f32 everywhere
   (`vec_t`, `vec3_t`, `tr.sinTable`, all `#[repr(C)]` fields); nothing
   crosses the wire wider; simulation code already referee-gated under
   ruling 12 is untouched.
2. **Known, unmeasurable divergence from retail Windows** (~1 ulp in the
   final f32 at `M_PI` sites — sin-table entries, deform/wave/turb angles,
   Lanczos3 weights). Recorded here instead of per-site notes.
3. Normalized at ratification: `tr_shade_calc.rs` bulge offset,
   `tr_image.rs` `Lanczos3` (+ `M_PI_OVER_3 = M_PI / 3.0f`, itself f64 by
   promotion). The canonical `native_math::qmath` note already calibrated
   to the math.h double; this DEC makes it citable. Applies as-is to the
   coming cgame/`cl_*` waves.

## DEC-42 — R3 registry/carrier design triplet (sit-downs, 2026-07-27)

Three user-settled design points closing the waves 7-13 review's open tail:

1. **`Arena::reset(slot0)`** is the registry teardown for `R_Init`'s
   memset-rebuild: every slot > 0 vacates into the free list with its
   generation bumped (all pre-reset handles go stale, ruling 11); slot 0's
   value is replaced in place at generation 0, keeping `Handle::slot_zero()`
   the persistent default identity across lives (A12). Capped arenas only —
   the image arena's purge stays `R_DeleteTextures`.
2. **Slot = index.** The arena slot number IS Raven's `shader_t::index` /
   `tr.shaders[]` int: disk-image pokes write `handle.index() as i32`, and
   bare-int consumers resolve through `Arena::handle_at_slot(u32)` (the
   slot's current generation). Sound because shaders are append-only in the
   oracle (no individual removal; whole-registry purge = reset, which
   preserves slot 0 and never renumbers). Overflow semantics align free:
   slot 0 = index 0 = `tr.defaultShader`.
3. **`EngineHostView` is the client track's engine carrier.** The
   client-only model family (`RE_RegisterModel`/`_Actual`,
   `r_load_mdxa`/`mdxm`/`md3`, `re_register_models_malloc`) re-signs onto
   `view: &mut EngineHostView` — `common` reached as `view.common` by
   sequential reborrow, server-shared helpers still callable (the view
   implements `EngineHost`), `RenderModels` via the ruled scoped slot-cast.
   The dedicated subset (`server_load`/`server_skins`, generic
   `impl EngineHost`) is untouched. **Follow-up recorded on #46 (user):**
   the cgame scoping sit-down revisits the whole client engine-carrier
   story (cl_* island, cgame vmcalls) and confirms or amends this
   convention before cgame waves start.
