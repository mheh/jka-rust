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
