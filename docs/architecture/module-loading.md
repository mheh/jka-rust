# Module Loading Design
Status: FROZEN (user sign-off 2026-07-03)     Supersedes: none
Decision prefix: LOAD     Ledger deps: DEC-05, DEC-07, DEC-08, DEC-09

## Standing context

Links only — never restated here:

- `docs/workspace-architecture.md` — crate graph and tiers (`abi-transport`,
  `crates/{jampgame,cgame,ui,jagame}` shells, `crates/{mp,sp}/{game,cgame,ui}`
  logic crates, `crates/native/platform`).
- `docs/porting-rules.md` — §A2 (faithful-first, deviate only behind green),
  §B3/§B4 (no hidden globals; state threaded), §B6 (single-singleton exception),
  §D11 (unsafe confined to the seam), §D12 (`#[repr(C)]` layout parity),
  `//TODO: Port <subject>` marker.
- `docs/decisions.md` — DEC-05 (module transport scope + `NativeDll | Static |
  Wasm`; retail-DLL hosting `i686-pc-windows`-only; WASM first-class, wasmtime
  after native parity), DEC-07 (SP cgame/ui via the vmachine shim), DEC-08
  (`Com_Error` recovery = typed panic + `catch_unwind`, scoped to the
  `ERR_DROP`/`ERR_DISCONNECT` caught escape — see LOAD-D11), DEC-09
  (verification layers).
- `docs/architecture/engine-seam.md` — the **typed call/dispatch** side (SEAM-D1
  compile-time-per-artifact vs runtime-per-module transport, SEAM-D4 pointer-word
  interpretation, `ModuleTransport` enum, `Execute`/`Dispatch`, `CEngine`,
  `SharedGameData`; **SEAM-D9/SEAM-D10** — the module shell's live
  `dllEntry`/`vmMain`/`GetGameAPI` + `ENGINE: OnceLock<CEngine>` live in each shell
  crate's `lib.rs`, `abi-transport`'s `entrypoints.rs` keeps only the raw C-ABI
  aliases, and the `qvm`/`sp_game` stubs are retired (2026-07-03 amendment), which
  this doc's LOAD-D4 reconciliation defers to; SEAM-D12 seam
  entrypoints/dispatchers are `extern "C-unwind"`, with the follow-up sweep of the
  `entrypoints.rs:9-27` raw aliases this doc relocates). This doc supplies the loader
  that produces the handles that doc dispatches through; the two are duals.
- `docs/architecture/two-island-model.md` — STATE-D3 (`extern "C-unwind"`).
- `docs/architecture/state-ownership.md` — STATE-D7 (resolving STATE-Q4): the
  receiverless leaf throw `pub fn com_error(level: ErrorLevel, msg: String) -> !`
  in `mp_engine_qcommon` (FROZEN in state-ownership § `com_init`/`com_frame`/
  `com_shutdown`/`com_error` entry points, cite `state-ownership.md:719`) that
  LOAD-D11 calls directly, and the `ErrorLevel` taxonomy — concretely the fatal
  variant `ErrorLevel::ERR_FATAL`, the ported `errorParm_t` enum's first member
  (`oracle/codemp/game/q_shared.h:451-457`; state-ownership STATE-D7 /
  lifecycle LIFE-D3 name the `ErrorLevel = errorParm_t` alias). This is the
  settled shape; lifecycle.md's earlier receiver-ful
  `com_error(engine, …)` is superseded by STATE-D7 and is **not** the one LOAD-D11
  uses.
- `docs/architecture/lifecycle.md` (FROZEN, 2026-07-03) —
  LIFE-D3 (`pub type ErrorLevel = errorParm_t`, `lifecycle.md:662`; per-mode MP-5 /
  SP-4 `errorParm_t`) and the module-load **trigger points**: the empty
  `ModuleRegistry` is default-constructed at `Com_Init` step-30 `VM_Init`
  (`lifecycle.md:187-215`) and hangs off `Engine.common.modules` (LIFE-Q5, shared
  with state-ownership STATE-D10); the game module is actually *loaded* at map spawn
  (`SV_SpawnServer → SV_InitGameProgs`, **post-Slice-0**), not at engine boot
  (`lifecycle.md:82,222`). lifecycle.md **punts `SV_InitGameProgs`'s module-load
  mechanics back to this doc** (`lifecycle.md:69-70`); the residual — the
  `SV_InitGameProgs`-equiv function's own crate/signature — is neither doc's settled
  territory (LOAD-Q12).
- `docs/abi-traps.md` — trap signatures. Which trap words are pointer-shaped (read
  through `WasmPtr<T>` in the `Wasm` dispatcher arm) is decided there + engine-seam
  SEAM-D4, **not** this doc; referenced only, never restated.

## Scope & non-goals

This doc freezes **how a module artifact is found, loaded, restarted, and
unloaded**, per transport and per mode: the `native/platform` loader mechanism
and its per-mode `ModuleSearchPolicy`; the DEC-05 drop-in parity matrix (both
directions); the SP three-way linkage (game / cgame / ui); the host-side
`WasmPtr<T>` pointer shape and wasm artifact production; and VM-restart /
unload semantics.

Non-goals (punted, each with its owning doc):

- **The typed call/dispatch mechanics** — `Execute`/`Dispatch`, the syscall
  word encoding, the `ModuleTransport` enum definition, the pointer-word
  *interpretation* inside a dispatcher → `docs/architecture/engine-seam.md`
  (SEAM-D3, SEAM-D4). This doc defines `WasmPtr<T>` and its accessors; that doc
  decides which trap words get read through them.
- **Lifecycle / boot / frame ordering** — *when* a load, restart, or unload
  fires during boot, connect, map change, or `vid_restart` →
  `docs/architecture/lifecycle.md` (**FROZEN, 2026-07-03**; it fixes the trigger
  points: the empty `ModuleRegistry` at `Com_Init` step-30 `VM_Init`, and the
  game-module load at map spawn via `SV_SpawnServer → SV_InitGameProgs`,
  post-Slice-0, `lifecycle.md:82,187-215,222`). This doc records the Raven
  create/destroy *cadence* only as ground truth for the restart semantics it
  freezes.
- **Which physical crate hosts each module shell** (`ENGINE` + live exports)
  and its dependency edges → engine-seam.md **SEAM-D10** (resolved upstream:
  `crates/jampgame` is the thin cdylib shell, deps `abi_transport` + `mp_game`).
  This doc anchors its Slice 0 call sites on that shell (Slice hooks); it does
  not restate the shell contract.
- **The `GetModuleAPI` OpenJK-native handshake contract** → SEAM-Q7 (engine-seam,
  open). This doc places it in the parity matrix; it does not define its body.
- **The raw inbound syscall trampoline** an engine hands a hosted DLL →
  engine-seam.md **SEAM-D11** (resolved upstream: one `extern "C-unwind"`
  C-shim trampoline per module slot reading a per-slot **injected** `EngineSlot
  { ctx, syscall }` — amended 2026-07-03, `engine-seam.md:545-570`).

## Raven ground truth

### MP native-DLL load chain

`vm_t *VM_Create( const char *module, systemCalls, vmInterpret_t interpret )`
(`oracle/codemp/qcommon/vm.cpp:471-472`) is the entry. It first guards
its parameters — `if ( !module || !module[0] || !systemCalls ) Com_Error(
ERR_FATAL, "VM_Create: bad parms" )` (`vm.cpp:480-482`), the earliest of the
function's two `ERR_FATAL`s. It then reuses a live slot whose stored `name`
matches (`if (!Q_stricmp(vmTable[i].name, module))`, `vm.cpp:485-489`), else
allocates the first free slot of `vmTable[MAX_VM]` — the free-slot test is
`!vmTable[i].name[0]` (`vm.cpp:493-494`), i.e. a slot's occupancy and identity
**are** its `name` field (`vm_local.h:119`, sitting in the same `vm_s` struct as
`dllHandle`/`entryPoint`, `:122-123`) — (`#define MAX_VM 3`, `vm.cpp:28-29`;
fatal if full, `vm.cpp:499-500`), stores the name and syscall (`Q_strncpyz(
vm->name, module, …)` `vm.cpp:505`, `vm->systemCall = systemCalls` `vm.cpp:506`),
skips the `fs_restrict` demo override (`vm.cpp:508-513` — forces `VMI_COMPILED`,
i.e. the QVM path, **out of scope DEC-05.4**; never gates the native path we host),
then attempts a native load: `Sys_LoadDll(module, &vm->entryPoint, VM_DllSyscall)`
(`vm.cpp:515-518`). Success returns immediately; failure falls through to the
QVM path — **out of scope (DEC-05.4)** (`vm.cpp:519-524`, non-fatal).

**Not-found disposition is caller-side and non-uniform (ground truth for
`load_module`'s `Option<SlotId>` return, LOAD-D11 amended 2026-07-03 —
resolving LOAD-Q10).** `VM_Create` **never fatals on load-not-found** — with the QVM
fallback out of scope, the observable native outcome is a `NULL` return, and
**each caller decides** the disposition, *differently per mode*:
`SV_InitGameProgs` → `if (!gvm) Com_Error(ERR_FATAL, "VM_Create on game failed")`
(`oracle/codemp/server/sv_game.cpp:1750-1752`, two lines below the
`VM_Create("jampgame", …)` call); `CL_InitCGame` → `Com_Error(ERR_DROP,
"VM_Create on cgame failed")` (`client/cl_cgame.cpp:1772-1774`); `CL_InitUI` →
`Com_Error(ERR_FATAL, "VM_Create on UI failed")` (`client/cl_ui.cpp:1479-1481`).
The split is **not uniform** — game and ui are `ERR_FATAL`, cgame is `ERR_DROP` —
so no single in-`VM_Create` fatal reproduces it; the fatal-vs-drop choice lives at
the caller.

`Sys_LoadDll` (Win32, `oracle/codemp/win32/win_main.cpp:811-887`):

1. **Filename** — `Com_sprintf(filename, ..., "%sx86.dll", name)`
   (`win_main.cpp:826`): `"jampgame"→jampgamex86.dll`, `"cgame"→cgamex86.dll`,
   `"ui"→uix86.dll`. No `ARCH_STRING` macro exists in the tree; the `x86` suffix
   is a hardcoded per-platform literal.
2. **Pure-server unpack** — `if (!Sys_UnpackDLL(filename)) return NULL;`
   (`win_main.cpp:849-852`), executed **before** any `LoadLibrary`.
   `Sys_UnpackDLL` (`win_main.cpp:762-800`) `FS_ReadFile`s the DLL (out of a pk3
   when the server is pure); if it is not in a pak it is used as-is
   (`FS_FileIsInPAK == -1` → `return true` without writing, `:774-779`, the
   non-pure case), otherwise the bytes are written back to disk
   (`FS_FOpenFileWrite`/`FS_Write`, `:781-796`) so `LoadLibrary` can open a real
   file. Any failure (`FS_ReadFile < 1`, can't open for write, short write)
   returns `false` → `Sys_LoadDll` returns `NULL` (non-fatal, falls to QVM).
   **MP-Win32-only:** no Unix (`unix_main.c`) or SP equivalent exists in oracle.
   Because it extracts from a pk3 and touches filesystem ownership, it is **in
   scope but deferred** to a post-B2 MP-server slice; Slice 0 stubs it
   (**LOAD-D7**).
3. **Search order**, first hit wins (`win_main.cpp:855-873`): bare
   `LoadLibrary(filename)` (CWD / default DLL search, `:855`); then
   `FS_BuildOSPath(fs_basepath, fs_game, filename)` (`:858-863`); then
   `FS_BuildOSPath(fs_cdpath, fs_game, filename)` **only if `fs_cdpath[0]`**
   (`:866-869`); else `return NULL` (`:871-873`). **No `fs_homepath`
   fallback.** `FS_BuildOSPath` builds `"<base>/<game>/<qpath>"`
   (`oracle/codemp/qcommon/files.cpp:479-498`). **Win32-only direct
   probe:** the bare `LoadLibrary(filename)` first step is Win32-specific — Unix
   `Sys_LoadDll` `#if 0`s its equivalent cwd `dlopen` (`unix_main.c:361-373`,
   *"do not load from installdir"*) and searches `fs_basepath/fs_game`
   (`unix_main.c:375-384`) then `fs_cdpath/fs_game` if `cdpath[0]`
   (`unix_main.c:391-396`) only, i.e. **no direct-first step**. The per-platform
   `direct_first` flag (Seam definition) carries this difference.
4. **Handshake** — two required exports: `dllEntry = GetProcAddress(lib,
   "dllEntry")` and `*entryPoint = GetProcAddress(lib, "vmMain")`; if either is
   null the library is freed and `NULL` returned; else `dllEntry(systemcalls)`
   hands the module the engine syscall trampoline (`win_main.cpp:879-887`). Unix
   mirrors this with `dlopen`/`dlsym` (`oracle/codemp/unix/unix_main.c:
   384,421,428,444`), filename `"%si386.so"` (`unix_main.c:346`; `ppc`/`axp`/
   `mips` variants, `-debug` infix in debug builds).

**macOS reality:** no `codemp/mac/` directory exists — **MP has no Mac
dynamic-loading backend in oracle** (dossier §3). SP's
`oracle/code/mac/mac_main.c:65-73` `Sys_LoadDll` is a hard-linked no-op
stub (`*entryPoint = vmMain; return (void*)1;`).

### MP create/destroy cadence (ground truth for restart, not lifecycle)

- **jampgame** — `SV_InitGameProgs` creates `VM_Create("jampgame", ...)`
  (`oracle/codemp/server/sv_game.cpp:1750`); `SV_ShutdownGameProgs`
  `VM_Call(GAME_SHUTDOWN); VM_Free(gvm)` (`sv_game.cpp:1666-1673`). Every normal
  map change **destroys+recreates**: `SV_ShutdownGameProgs()` (`sv_init.cpp:484`)
  then `SV_InitGameProgs()` (`sv_init.cpp:662`). Only `map_restart` uses the
  in-place `VM_Restart` via `SV_RestartGameProgs` (`sv_game.cpp:1712-1715`,
  `sv_ccmds.cpp:296`). Torn down for good at `SV_Shutdown` (`sv_init.cpp:946`).
- **cgame** — `CL_InitCGame` `VM_Create("cgame", ...)`
  (`oracle/codemp/client/cl_cgame.cpp:1771`); `CL_ShutdownCGame`
  `VM_Free(cgvm)` (`cl_cgame.cpp:601-603`). Destroyed+recreated on every map
  load via `CL_DownloadsComplete → CL_FlushMemory → CL_ShutdownAll /
  CL_InitCGame` (`cl_main.cpp:1497,1501`), and on `vid_restart`
  (`cl_main.cpp:1322,1324,1362`). Not freed by plain disconnect.
- **ui** — `CL_InitUI` `VM_Create("ui", ...)`
  (`oracle/codemp/client/cl_ui.cpp:1478`), version-checked against
  `UI_API_VERSION` (`cl_ui.cpp:1484-1487`); `CL_ShutdownUI` `VM_Free(uivm)`
  (`cl_ui.cpp:1444-1453`). **UI is NOT session-persistent:** `CL_FlushMemory`
  runs `CL_ShutdownAll` (frees `uivm`, clears `cls.uiStarted`,
  `cl_main.cpp:669,677`) then `CL_StartHunkUsers → CL_InitUI`
  (`cl_main.cpp:766,2471`), so it is destroyed+recreated on every map load in
  lockstep with cgame, plus on `vid_restart`.

`VM_Restart` (`vm.cpp:391-458`) documents *"DLL's can't be restarted in place"*
(`vm.cpp:398`): the native arm saves `systemCall`/`name`, `VM_Free(vm)`, then
`VM_Create(...)` — a full **destroy+recreate** disguised as a restart
(`vm.cpp:399-409`). The in-place data-segment reset arm (`vm.cpp:412-451`) is
QVM-only, out of scope. `VM_Free` unconditionally clears the **global**
`currentVM = lastVM = NULL` regardless of which slot was freed
(`vm.cpp:624-625`) — a shared-state clobber bug (dossier §1, design fork 6).

### SP three-way linkage

SP loads exactly one module. `Sys_GetGameAPI`
(`oracle/code/win32/win_main.cpp:478-547`) loads `"jagamex86.dll"`
(`:489`) — search is **narrower than MP's**: `<cwd>/<debugdir>/jagamex86.dll`
(`:515`) then `<cwd>/jagamex86.dll` (`:524`), no `fs_*` cvars; both fail →
`Com_Error(ERR_FATAL, "Couldn't load game")` (`:536`). Symbol:
`GetProcAddress(game_library, "GetGameAPI")` (`:540`), then `return
GetGameAPI(parms)` (`:546`). The game import/export is a **struct of typed
fn-pointers**, not numbered syscalls (`code/game/g_public.h`), version-checked
engine-side (`sv_game.cpp:682-684`).

Immediately after, `SV_InitGameProgs` fakes `VM_Create("cl")`
(`code/server/sv_game.cpp:676-679`) — **not a second DLL**. That is SP's
fake-VM shim `oracle/code/client/vmachine.h:72-84`: for module `"cl"` it
calls `Sys_LoadCgame(&cgvm.entryPoint, VM_DllSyscall)`, which
`GetProcAddress`es `dllEntry`/`vmMain` on the **same** `game_library` handle
already holding `jagamex86.dll` (`win_main.cpp:557-570`) — cgame logic is
compiled *into* `jagamex86.dll` (`code/game/game.vcproj`). Console/Mac builds
resolve the same `GetGameAPI`/`vmMain`/`cg_dllEntry` as plain `extern` symbols
with zero `LoadLibrary` (`win_main_console.cpp:542-567`, `mac_main.c:35-73`).
**UI is always statically linked into the exe in every SP build** — no UI DLL
project exists anywhere in SP. Conclusion (dossier §4): `jagamex86.dll` is the
only ever-loadable SP module; cgame is never its own DLL; UI is never a module.
Our port constructs **no** SP `ModuleSearchPolicy` (LOAD-D5, DEC-07): `jagame` is
reached via the `GetGameAPI` table factory, so this SP search order is recorded
here as ground truth **only** — its loader surface is dropped (porting-rules §20;
LOAD-D1 round-3 amendment, which removes the SP policy value from the Seam).

### OpenJK ABI divergence scope (for the parity matrix)

`tools/closure-prototype/NOTES.md:60-68` records OpenJK diverges from Raven 1.01
in **game-private structs only** (`gclient_s`, `clientPersistant_t`,
`clientSession_t` differ in size); **ABI-crossing structs are unchanged**
(`playerState_t` 1552 B, `usercmd_t` 28 B, `gameState_t`), and no divergence
exists for trap numbers, syscall-table shape, `vmMain`, or `dllEntry`. Loading a
Rust module under real OpenJK is therefore safe at the seam but game-private
layout must be checked against the actual host binary, not just the oracle
(`NOTES.md:65-68`).

## State ownership

Loading-specific globals (the dispatch-side `syscall` pointer, `ENGINE`
`OnceLock`, `cgvm.entryPoint`, and the eliminated `currentVM`/`lastVM`) are owned
by `engine-seam.md`'s table and only cross-referenced here to avoid duplication
(porting-rules §4 / doc-standards §4). SEAM-D11's per-slot engine trampoline cell
is physically composed into the `ModuleSlot.engine` field this doc's registry owns
(LOAD-D8 round-3 amendment); its **type/shape** stays owned by engine-seam
SEAM-D11, which — as **amended 2026-07-03 (load-time injection, Raven-style)**,
`engine-seam.md:545-570` — freezes it as the struct
`EngineSlot { ctx: *mut c_void, syscall: SlotSyscall }` (`engine-seam.md:547`) in
this same `mp_engine_qcommon` crate, where
`SlotSyscall = extern "C-unwind" fn(ctx: *mut c_void, args: *const isize) -> isize`
(`engine-seam.md:545-546`). The field type is therefore spelled **`EngineSlot`**
everywhere in this doc (mechanical unification, 2026-07-03) — the sole *defined*
type in the owner, named directly, **not** an opaque placeholder.

**LOAD-Q11 is resolved by the injection (see LOAD-D8's 2026-07-03 amendment).**
The round-4 hole — that the below-facade `mp_engine_qcommon` could not name the
`mp_engine_core::Engine` aggregate its `Cell<*mut Engine>` field pointed at
without crossing a **forbidden uphill edge** (`mp_engine_core` depends *down* on
`mp_engine_qcommon`, workspace-architecture § Dependency edges) — is dissolved by
storing **injected** state instead of a typed `*mut Engine`. Mirroring Raven's
`VM_Create`, which **receives** its `systemCalls` argument (`vm.cpp:471-472`) and
stores it (`vm->systemCall = systemCalls`, `vm.cpp:506`) rather than naming the
server, the injected `EngineSlot { ctx: *mut c_void, syscall: SlotSyscall }`
carries an **opaque** `ctx` pointer plus the syscall fn pointer handed in at
module-load time — so `mp_engine_qcommon` **never names** `mp_engine_core::Engine`
or `mp_engine_server::sv_game_system_calls` (both uphill), and no crate-graph edge
is added (`engine-seam.md:545-570`, the SEAM-D11 amendment). The construction and
initialisation of the cell that was previously unresolved is now settled: it is
built at `load_module` from that function's own injected parameters (LOAD-D8's
2026-07-03 amendment), not from an `Engine` in scope. This changes none of
this doc's slot *semantics*; it removes the compilability blocker LOAD-Q11 named.

| Raven global | oracle cite | Rust owner (crate::Type.field) | constructed by | threaded via |
|---|---|---|---|---|
| `vmTable[MAX_VM]` (`MAX_VM = 3`) | `qcommon/vm.cpp:28-29` | `mp_engine_qcommon::ModuleRegistry.slots: [Option<ModuleSlot>; MAX_VM]` — **per-slot only, no current-module global** (LOAD-D5); home crate `mp_engine_qcommon` mirrors `vm.cpp`'s subsystem (LOAD-D8; engine-seam.md state table — `ModuleRegistry` mirrors oracle `qcommon/vm.cpp`) | `ModuleRegistry::load_module` (FROZEN — LOAD-D8: slot reuse-by-name `vm.cpp:485-489`, first free slot `vm.cpp:494`, `Com_Error(ERR_FATAL)` when all `MAX_VM` full `vm.cpp:499-500`) | **container owned at `Engine.common.modules`** (state-ownership STATE-D10 / lifecycle LIFE-Q5), default-constructed empty at `Com_Init` step-30 `VM_Init` (`lifecycle.md:187-215`), reached as `engine.common.modules`; dispatcher `engine` arg (engine-seam) |
| `vm->dllHandle` | `qcommon/vm_local.h:111-146` | `ModuleSlot.module.lib: libloading::Library` (LoadedModule inside the slot) | `sys_load_dll` | held in slot; dropped on unload |
| `vm->entryPoint` | `qcommon/vm_local.h:123` | `ModuleSlot.module.entry: RawVmMain` (LoadedModule inside the slot) | handshake in `sys_load_dll` | passed to `Dispatch` (engine-seam) |
| `vm->name` (slot identity for reuse/free) | `qcommon/vm_local.h:119`; set `vm.cpp:505`, read `vm.cpp:486,494` | `ModuleSlot.name: String` (LOAD-D8 round-3 amendment — the composed slot: `name` sits on the slot beside its `module`/`engine`, the faithful `vm_s` mirror) | stamped by `load_module` from its `name: &str` arg | held in slot; `load_module`'s reuse scan compares it (`vm.cpp:485-489`) |
| per-slot injected engine cell (SEAM-D11) | engine-seam SEAM-D11 (supplants `currentVM`, `qcommon/vm.cpp:24`) | `ModuleSlot.engine: EngineSlot` — the concrete type frozen by engine-seam SEAM-D11 (amended 2026-07-03) as `struct EngineSlot { ctx: *mut c_void, syscall: SlotSyscall }` (`engine-seam.md:547`), in this same `mp_engine_qcommon` crate. Composed into this doc's slot per LOAD-D8 round-3 | **injected at `load_module`** from its `system_calls: SlotSyscall` + `ctx: *mut c_void` params (Raven `VM_Create` receives+stores `systemCalls`, `vm.cpp:471-472,506`; the retired `EngineSlot::enter` per-call cell is superseded — `engine-seam.md:545-570`) | read only by that slot's raw C-shim syscall trampoline (engine-seam SEAM-D11) |
| `currentVM`, `lastVM` | `qcommon/vm.cpp:24-25` | **eliminated** (LOAD-D5; owned/justified in engine-seam state table) — the `VM_Free` clobber (`vm.cpp:624-625`) is structurally unreproducible | — | — |
| `gvm` (jampgame slot) | `server/sv_game.cpp:1750` | `ModuleRegistry` slot; server state holds the `SlotId` | `SV_InitGameProgs`-equiv | server state |
| `cgvm`, `uivm` (MP slots) | `client/cl_cgame.cpp:1771`, `client/cl_ui.cpp:1478` | `ModuleRegistry` slots; client state holds their `SlotId`s | `CL_InitCGame` / `CL_InitUI`-equiv | client state |
| SP `game_library` `HINSTANCE` | `code/win32/win_main.cpp:459` | **no jka-rust owner** — our SP is fully static (DEC-07) → no handle; a retail SP `game_library` only exists when hosting a real SP game DLL, which is outside DEC-05.3 (MP `jampgamex86.dll` replacements only) and has no parity-matrix row | `Sys_GetGameAPI`-equiv | — |
| SP `cgvm`/`uivm` (vmachine shim) | `code/client/vmachine.h:54-55` | static dispatch handles, **no load** (DEC-07); owned as `cgvm.entryPoint` in engine-seam state table | static link | dispatcher arg (engine-seam) |

**Where the `ModuleRegistry` container hangs (settled upstream, cross-ref only).**
The registry this doc's slot API operates on is a **field of `Common`** —
`Engine.common.modules: ModuleRegistry` (state-ownership **STATE-D10** / lifecycle
**LIFE-Q5**, resolved 2026-07-02, shared) — mirroring Raven's `vmTable` being a
`qcommon`-subsystem file-scope static (`vm.cpp:28-29`); `Common` and
`ModuleRegistry` are both `mp_engine_qcommon`, so no cross-crate edge is added. It
is default-constructed **empty** at `Com_Init` step-30 `VM_Init`
(`lifecycle.md:187-215`) and reached as `engine.common.modules`; a boot/spawn call
site invokes `.load_module(...)` on it. This doc owns the registry's *type/shape*
(LOAD-D8); its *attachment field* is STATE-D10 / LIFE-Q5, cited not restated
(doc-standards §4). `lifecycle.md`'s own restatement of that shape
(`lifecycle.md:199`) already reads `{ slots: [Option<ModuleSlot>; MAX_VM] }`,
matching this doc's frozen LOAD-D8 shape — the sibling is consistent, no
reconciliation is owed.

## Seam definition

Exact signatures below are FROZEN; porters fill bodies without changing them.
Everything here is host-side / engine-side (native track, porting-rules §E) —
none of it crosses the ABI as a `#[repr(C)]` type except `WasmPtr<T>`'s target
loads, which reuse the already-asserted module structs.

What freezes here is the owning **crate** and the exact pub **signatures**
(doc-standards §5). The seven loader items below (`ModuleNaming`,
`ModuleSearchPolicy`, `SearchStep`, `LoadedModule`, `sys_load_dll`,
`unload_module`, `RestartKind`) consolidate Raven's per-platform `Sys_LoadDll`
(`win_main.cpp:811-887`, `unix_main.c:346-444`) and `VM_Create`/`VM_Restart`
(`vm.cpp:471-472,391-458`) into the single LOAD-D1 loader — and *because* these
types synthesize `vm.cpp` + `win_main.cpp` + `unix_main.c`, **no** single owning
Raven-header folder applies. LOAD-D6 pins their file tree under
`crates/native/platform/src/module_loader/` — **mechanical, not architectural**
(one-type-per-file, porting-rules; the mapping is fixed only so a porter and the
dry-run land on the same paths):

```
crates/native/platform/src/module_loader/
  naming.rs         // ModuleNaming
  search_policy.rs  // ModuleSearchPolicy
  search_step.rs    // SearchStep
  loaded_module.rs  // LoadedModule
  restart_kind.rs   // RestartKind
  loader.rs         // sys_load_dll + unload_module
```

`LoadedModule`'s fields (`lib`/`entry`) are set by `sys_load_dll` in the sibling
`loader.rs`, so they take **`pub(crate)`** visibility (LOAD-D12f) — reachable
across the one-type-per-file split within `native/platform`, no wider. Ordinary
Rust idiom for the split (porting-rules §D12: internal-only types get idiomatic
shape); pinned only so porters spell it uniformly. (`name` is no longer a
`LoadedModule` field — it moved up to the composed `ModuleSlot`, LOAD-D8 round-3.)

The raw ABI aliases relocated by LOAD-D6 are **not** loader types, so they do not
sit under `module_loader/`; they land in a sibling file that mirrors the
`abi-transport` source they move from (LOAD-D12e — mechanical, same one-type-block
/ mirror-the-source rationale, pinned only so a porter and the dry-run agree):

```
crates/native/platform/src/
  entrypoints.rs    // AbiCommand, AbiWord, RawSyscall, RawDllEntry, RawVmMain
                    //   (LOAD-D6); mirrors abi-transport src/entrypoints.rs;
                    //   abi-transport re-exports all five from its own entrypoints.rs
```

**Repo reconciliation — what stays in `abi-transport` (no new fork).** The live
`crates/abi-transport/src/entrypoints.rs` also declares four table-handshake
aliases — `RawImportTable`, `RawExportTable`, `RawGetModuleApi`, `RawGetGameApi` —
and two `#[no_mangle]` stub-export modules (`qvm`: `dllEntry`/`vmMain`/`GetModuleAPI`;
`sp_game`: `GetGameAPI`). Neither is loader vocabulary, so **LOAD-D6 does not
relocate them** — only the five aliases above move. The four table aliases **stay
in `abi-transport`**; `AbiCommand` (relocated) stays referenceable there through
LOAD-D6's re-export, so `RawGetModuleApi`'s signature still typechecks in place. The
`qvm`/`sp_game` stub modules are **retired** — superseded **not** by LOAD-D4's
original placement text but by engine-seam **SEAM-D9/SEAM-D10** (2026-07-03 amendment,
checkpoint-3 finding 14, `engine-seam.md:589-597`): their `#[no_mangle]` symbols
collide at cdylib link with the live per-shell exports SEAM-D9/D10 mandate, so they
cannot coexist with them (a link error, not a choice). This doc's own Scope punts
**which physical crate hosts each module shell (`ENGINE` + live exports)** to SEAM-D10
(§ Scope & non-goals), so that placement is engine-seam's: the live
`dllEntry`/`vmMain`/`GetGameAPI` and the `ENGINE: OnceLock<CEngine>` live in each
**shell crate's `lib.rs`** (`crates/jampgame/src/lib.rs`, …), **not** in
`abi_transport::entrypoints::native` (`engine-seam.md:574-597,647-663`); the
`GetModuleAPI`/`GetGameAPI` **bodies** stay SEAM-Q7-owned. `abi-transport`'s
`entrypoints.rs` keeps **only** the raw C-ABI type aliases — the four table aliases
above plus LOAD-D6's five relocated aliases' re-exports (`engine-seam.md:879-880`).
This applies LOAD-D6 (move scope) + SEAM-D9/D10 (shell-crate export placement) — it
settles nothing new; a porter neither leaves the four table aliases dangling nor keeps
`qvm`/`sp_game`. (Reconciliation note: LOAD-D4's original "places … in the `native`
arm" wording is retracted — see the LOAD-D4 amendment in § Per-target entrypoint
modules and § Decisions; engine-seam SEAM-D9/D10 wins by this doc's own Scope punt.)

### The loader mechanism — `crates/native/platform`

One `libloading`-based loader executes a mode-supplied policy value (LOAD-D1).
`libloading` and all OS types are confined to this crate (native-only, never in
a module logic crate — LOAD-D4).

```rust
// crates/native/platform — the loading MECHANISM (LOAD-D1). One impl.

/// Per-platform artifact naming. Windows/Unix entries are faithful to oracle;
/// the macOS `.dylib` entry is our documented extension (no oracle precedent —
/// MP has no Mac loader, dossier §3). Exact macOS base string: LOAD-Q1.
pub struct ModuleNaming {
    /// Appended to the bare module name, e.g. "x86.dll" → "jampgamex86.dll".
    /// win_main.cpp:826 (`Some("x86.dll")`); unix_main.c:346 (`Some("i386.so")`).
    /// `None` = suffix not yet resolved — the macOS arm only (LOAD-Q1; oracle has
    /// no Mac MP loader, dossier §3). Widened to `Option` 2026-07-03 (mechanical)
    /// so "unset" is representable without a placeholder literal.
    pub suffix: Option<&'static str>,
}

/// A mode's search policy — a value, built **per load by the caller in
/// `mp_engine_qcommon`** (LOAD-D9), never by `native/platform` (which stays
/// cvar-free, porting-rules §B3). All paths are already resolved when built.
pub struct ModuleSearchPolicy {
    pub naming: ModuleNaming,
    /// Bare `LoadLibrary(filename)` / CWD-default probe tried first (MP Win32
    /// only, win_main.cpp:855; Unix MP `#if 0`s its cwd dlopen, unix_main.c:361-373,
    /// so its policy sets this `false`).
    pub direct_first: bool,
    /// Ordered probes after the direct one; first hit wins.
    pub steps: Vec<SearchStep>,
}

pub enum SearchStep {
    /// `FS_BuildOSPath(base, gamedir, filename)` (files.cpp:479-498). Carries
    /// **resolved** values (LOAD-D9), NOT cvar names: the caller in
    /// `mp_engine_qcommon` reads `Cvar_VariableString("fs_basepath")` /
    /// `("fs_cdpath")` / `("fs_game")` (win_main.cpp:858-860) once B1 lands and
    /// plants the results here, so `native/platform` never touches a cvar table.
    /// An empty-`fs_cdpath` step is **omitted by the caller** (LOAD-D9 round-3
    /// amendment), so every step handed to `sys_load_dll` is real and it walks
    /// them blindly. MP order = basepath/fs_game then cdpath/fs_game; NO homepath
    /// (win_main.cpp:858-869).
    FsPath { base: PathBuf, gamedir: String },
    // NOTE: the SP `CwdRelative { subdir }` variant (win_main.cpp:515,524) is
    // dropped — our SP constructs no policy (LOAD-D1 round-3 / LOAD-D5 / DEC-07),
    // so it was zero-constructor surface (porting-rules §20). SP's `cwd/<debugdir>`
    // then `cwd` order lives in `## Raven ground truth` only.
}
```

The per-load `ModuleSearchPolicy` **value** is built by the **caller in
`mp_engine_qcommon`** (LOAD-D9), at the `ModuleRegistry::load_module` call site —
not by `native/platform`, which stays cvar-free (porting-rules §B3/§B4). Once the
cvar port (B1) lands the caller resolves `Cvar_VariableString("fs_basepath")` /
`("fs_cdpath")` / `("fs_game")` (`win_main.cpp:858-860`) and plants the
**resolved** `FsPath { base, gamedir }` values; Slice 0 (pre-B1) builds the same
value from hardcoded / CLI paths. This doc freezes the per-mode policy **value** —
its `naming` / `direct_first` / `steps` — which is what Slice 0 needs; *when* that
construction runs is `lifecycle.md`'s — settled there as **map spawn**
(`SV_SpawnServer → SV_InitGameProgs`, post-Slice-0), not engine boot
(`lifecycle.md:82,222`). This construction is **inline at the load call site**
(the `SV_InitGameProgs`-equiv; its *trigger* is settled by lifecycle, but its own
crate/signature is unpinned — **LOAD-Q12**; see Slice hooks, *Referenced but owned
elsewhere*), **not** a separately-homed frozen helper this doc pins: the block below
is the illustrative body that call site runs, landing wherever the call site lands,
so a porter does **not** invent a home or a function name for it. The MP
`direct_first` split is per-platform (LOAD-D1, amended 2026-07-02):

```rust
// mp_engine_qcommon caller, per load (LOAD-D9): resolve the cvars, then build the value.
// Win32 (win_main.cpp:855-869): LoadLibrary-direct, then basepath/fs_game, then cdpath/fs_game.
// Unix  (unix_main.c:361-396):  basepath/fs_game then cdpath/fs_game; the cwd/installdir
//   dlopen is #if 0'd ("bk010205 - do not load from installdir", unix_main.c:373) → NO direct probe.
let gamedir  = cvar_string("fs_game");            // win_main.cpp:860 (Slice 0: CLI/hardcoded)
let basepath = cvar_string("fs_basepath");        // :858 (resolved install path, never empty)
let cdpath   = cvar_string("fs_cdpath");          // :859
let mut steps = vec![
    SearchStep::FsPath { base: basepath.into(), gamedir: gamedir.clone() }, // :862-863 (attempted unconditionally)
];
// Reproduce Raven's `if (cdpath[0])` guard (win_main.cpp:866 / unix_main.c:391) at
// the CONSTRUCTION site: the caller OMITS the cdpath step when fs_cdpath is empty,
// so the policy handed to sys_load_dll holds only real steps and the loader walks
// them blindly (LOAD-D9 round-3 amendment — native/platform stays cvar-semantics-free).
if !cdpath.is_empty() {
    steps.push(SearchStep::FsPath { base: cdpath.into(), gamedir: gamedir.clone() }); // :866-869
}
ModuleSearchPolicy {
    naming: ModuleNaming { suffix: /* Win32 Some("x86.dll") | Unix Some("i386.so") |
                                       macOS None until LOAD-Q1 resolves the literal
                                       (macOS arm not built for Slice 0's win/linux targets) */ },
    // Win32: true (win_main.cpp:855); Unix: false (unix_main.c:361-373 is #if 0'd).
    direct_first: cfg!(windows),
    steps,
}
// SP constructs NO policy: our SP never exercises the loader (jagame via the
// GetGameAPI table factory; retail SP-DLL hosting not ported — LOAD-D1 round-3,
// LOAD-D5, DEC-07). SP's `cwd/<debugdir>` then `cwd` search order
// (win_main.cpp:515,524) is documented under `## Raven ground truth` only
// (porting-rules §20: zero-caller surface dropped with a note).
```

### Load / unload / restart

```rust
/// A live native module: the library handle + its resolved `vmMain` entrypoint,
/// held inside a `ModuleSlot` in the `ModuleRegistry` (State ownership). The bare
/// name it was loaded under and the SEAM-D11 engine cell live on the owning
/// `ModuleSlot`, not here (LOAD-D8 round-3 amendment — the composed-slot shape;
/// `vm_s`'s `name`/`dllHandle`/`entryPoint` are mirrored across `ModuleSlot`+its
/// `LoadedModule`, vm_local.h:119,122-123).
pub struct LoadedModule {
    pub(crate) lib: libloading::Library,   // vm->dllHandle (win_main.cpp:855-863); pub(crate) per LOAD-D12f
    pub(crate) entry: RawVmMain,           // "vmMain" (win_main.cpp:880); RawVmMain defined here in
                                           // native/platform (LOAD-D6), re-exported by abi-transport
}

/// Faithful to Sys_LoadDll (win_main.cpp:811-887) MINUS the pure-server
/// `Sys_UnpackDLL` pre-step (`:849-852`), which is IN SCOPE but DEFERRED to a
/// later MP-server slice (LOAD-D7 — it needs FS_ReadFile/FS_FileIsInPAK/
/// FS_FOpenFileWrite, which land with the filesystem port B2). Slice 0 is
/// non-pure and stubs it: where the pre-step would run the porter leaves a
/// `//TODO: Port Sys_UnpackDLL` + `// Source:` marker (porting-rules §14 —
/// unported deps are explicit, never a silent no-op), NOT a silently-swallowed
/// step. Namely: apply naming, then walk `policy.steps` **in order, blindly** —
/// the caller has already omitted any empty-base step at construction (LOAD-D9
/// round-3 amendment), so `sys_load_dll` stays cvar-semantics-free and simply
/// tries every step it is handed, first hit wins. At a hit, `GetProcAddress`/
/// `dlsym` "dllEntry"+"vmMain" (both required) and call `dllEntry(syscall)`,
/// returning `LoadedModule { lib, entry }`. The slot's reuse-by-name key
/// (`ModuleSlot.name`) is stamped by the registry (`load_module`), not here — the
/// `name` arg is consumed only for filename synthesis, never for slot identity.
/// `None` = not found (Raven's QVM fallback is out of scope, DEC-05.4; the caller
/// decides fatal-vs-skip per mode).
pub fn sys_load_dll(policy: &ModuleSearchPolicy, name: &str, syscall: RawSyscall)
    -> Option<LoadedModule>;   // RawSyscall/RawVmMain defined in this crate (LOAD-D6)

/// Faithful to Sys_UnloadDll via VM_Free (vm.cpp:605-610): drop the library,
/// clearing the slot. No global `currentVM`/`lastVM` clobber (LOAD-D5).
pub fn unload_module(module: LoadedModule);

/// VM-restart semantics (LOAD-D2). NativeDll and Static restart ONLY by
/// drop+recreate — Raven's actual native path (vm.cpp:398-409). There is NO
/// in-place native reset (the QVM in-place arm is out of scope).
pub enum RestartKind {
    /// unload_module(old) then sys_load_dll(...) — native map-change path
    /// (sv_init.cpp:484,662) and Raven's native VM_Restart (vm.cpp:399-409).
    DropRecreate,
    /// Wasm-only fast path (LOAD-D2): reset linear memory to the initial data
    /// image + re-run module init, WITHOUT re-instantiating. Must be observably
    /// identical to DropRecreate (parity test, Verification strategy).
    WasmInPlaceReset,
}
```

**Amendment (2026-07-03, user ruling item 18 — Unix in-loader fatal on a missing
export, release builds).** The `sys_load_dll` handshake step above ("`dlsym`
'dllEntry'+'vmMain' (both required)") maps a **found library whose required exports
are missing** to `None` — the debug-build behavior. Raven's Unix loader, however, is
**per-build**: on a missing `vmMain`/`dllEntry` it does `#ifdef NDEBUG` →
`Com_Error(ERR_FATAL, "Sys_LoadDll(%s) failed dlsym(vmMain): …")`, `#else` →
`Com_Printf(…)` then `return NULL` (`oracle/codemp/unix/unix_main.c:431-436`;
note the semantics — `NDEBUG` = **release** takes the fatal, debug takes the print).
The port reproduces this faithfully (porting-rules §20 — preserve per-mode quirks):
the missing-export arm gains a `cfg(not(debug_assertions))` branch that raises the
receiverless `com_error(ErrorLevel::ERR_FATAL, …)` dual, while the
`cfg(debug_assertions)` branch keeps `com_printf` + `None` (the current text). The
**`Option` contract is otherwise unchanged**: `None` still means "not found on any
policy step", and caller dispositions (`SV_InitGameProgs`/`CL_InitUI` `ERR_FATAL`,
`CL_InitCGame` `ERR_DROP`) are unchanged — the release fatal fires only for the
distinct *found-but-no-vmMain* case, before any caller-side `None`-match.

**Open sub-question (mechanism, tracked — NOT settled; owner: slice-0 wiring).** The
*contract* above is user-settled; the *mechanism* is not. `sys_load_dll` lives in
`crates/native/platform` (tier −1), which **cannot name `mp_engine_qcommon::com_error`
(an uphill edge — the same class as the checkpoint-2 `EngineSlot`-injection fix)**. So
where the `cfg(not(debug_assertions))` fatal physically fires is open, with candidates:
(a) an **injected fatal hook** passed into the loader at load time (à la the settled
`systemCalls`/`ctx` injection); (b) a **platform-local sys-fatal** primitive in
`native/platform` (print+`exit`, no `com_error`); or (c) a **three-state loader return**
(found / not-found / found-but-bad-export) with the cfg-gated `com_error` raised by the
caller in `mp_engine_qcommon::load_module`. **Do not pick one** — flagged for slice-0
wiring (checkpoint-5 finding 23). This does not change the frozen `sys_load_dll`
signature; it only defers where the release-fatal is emitted.

**Alias home (LOAD-D6).** The raw ABI aliases named above — `RawVmMain`
(`LoadedModule.entry`), `RawSyscall` (`sys_load_dll`'s `syscall`), and
`RawDllEntry` (the handshake call target `dllEntry(syscall)`,
`win_main.cpp:879-887`) — **live in `crates/native/platform`**, specifically in
`crates/native/platform/src/entrypoints.rs` (LOAD-D12e), a base tier: they
are Raven-free platform-ABI vocabulary; `RawDllEntry` travels with its two
siblings under LOAD-D6's same rationale — all three already co-reside in the
`crates/abi-transport/src/entrypoints.rs` block LOAD-D6 relocates. The two
supporting aliases `AbiCommand` and `AbiWord` (`entrypoints.rs:3-4`) — which
`RawVmMain`'s signature is written in terms of (its body at `entrypoints.rs:10-24`
names both) — **relocate with them**: leaving them in `abi-transport` would force
the forbidden `native/platform → abi-transport` uphill edge just to typecheck
`RawVmMain`, so all **five** aliases move as one block (LOAD-D6, amended below).
`abi-transport` takes a downhill dep on `native/platform` and **re-exports** them
from `entrypoints.rs`, so existing `abi-transport` consumers are unaffected and no
tier inversion occurs. The frozen loader signatures therefore compile in
`native/platform` with no new outward edge. `workspace-architecture.md` gains the
`abi-transport → native/platform` edge (updated there separately).

**Cargo wiring (mechanical — realizes LOAD-D1 + LOAD-D6, restated by LOAD-D12d,
not a new decision).** Both crates are currently `[dependencies]`-less; a porter
adds exactly two edges and no others: `crates/native/platform/Cargo.toml` gains a
`libloading` dependency — used only by `module_loader/loader.rs`'s `sys_load_dll`
(the LOAD-D1 loader mechanism, confined to this crate per LOAD-D4) — and
`crates/abi-transport/Cargo.toml` gains a `native_platform` path dependency (the
LOAD-D6 downhill re-export edge, already documented in
`workspace-architecture.md`). Crate names per their `Cargo.toml`: `native_platform`,
`abi_transport`. (The wasm-host crate's own Cargo wiring — the `wasmtime` dep and
`mp_engine_qcommon`'s feature-gate onto it — is LOAD-D10 / `workspace-architecture.md`.)

The three aliases are **not new shapes** — they are the existing
`entrypoints.rs` definitions verbatim in **arg list, types, and arity**;
`verbatim` is scoped to those, **not** to the calling convention. Their calling
convention is `extern "C-unwind"` per **engine-seam SEAM-D12** (which adopts
STATE-D3: a `Com_Error` panic, DEC-08, must traverse a real host's live C frames
mid-trap — plain `extern "C"` aborts on unwind). The live
`crates/abi-transport/src/entrypoints.rs:9-27` aliases are **still plain
`extern "C"`** (pre-STATE-D3, unused); SEAM-D12 records flipping them to
`extern "C-unwind"` as an **explicit follow-up slice task** — this doc therefore
**cross-refs that sweep rather than asserting current-repo agreement** (LOAD-D12a).
The relocation under LOAD-D6 lands the aliases already spelled `extern "C-unwind"`;
the difference vs the stale repo spelling is unwind-behavior only (same C ABI /
symbol shape), not a layout/arity change:

```rust
// Source: crates/abi-transport/src/entrypoints.rs:3-24 (relocated under LOAD-D6).
pub type AbiCommand  = core::ffi::c_int;         // vmMain command selector
pub type AbiWord     = isize;                     // pointer-width trap word (SEAM-D4)
pub type RawSyscall  = *const core::ffi::c_void;  // opaque VM_DllSyscall trampoline
pub type RawDllEntry = extern "C-unwind" fn(syscall: RawSyscall);   // "dllEntry"
pub type RawVmMain   = extern "C-unwind" fn(      // "vmMain": command + 12 arg words
    AbiCommand,
    AbiWord, AbiWord, AbiWord, AbiWord, AbiWord, AbiWord,
    AbiWord, AbiWord, AbiWord, AbiWord, AbiWord, AbiWord,
) -> AbiWord;
```

The arity worry does not arise: oracle's untyped-variadic `entryPoint`
(`int (QDECL *)(int, ...)`) is modeled as the **opaque** `RawSyscall`
(`*const c_void`) that the module casts — no arity is exposed. The module-facing
`RawVmMain` arity (command + 12 words) is the already-frozen SEAM-D4 word ABI in
`entrypoints.rs`, not a fresh choice. With these definitions reachable in
`native/platform`, `LoadedModule.entry: RawVmMain` and `sys_load_dll(…, syscall:
RawSyscall)` name local types and compile as written.

### Module registry — `crates/mp/engine/qcommon`

`sys_load_dll` / `unload_module` above are the per-artifact primitives. The slot
registry that owns them — the `vmTable[MAX_VM]` replacement — is FROZEN (LOAD-D8)
against `VM_Create`'s slot semantics; its home crate `mp_engine_qcommon` mirrors
`vm.cpp`'s subsystem (engine-seam.md state table — `ModuleRegistry` mirrors oracle
`qcommon/vm.cpp`).

```rust
// crates/mp/engine/qcommon — the slot registry (LOAD-D8), replaces vmTable[MAX_VM].
pub struct SlotId(pub(crate) u32);   // index into slots[0..MAX_VM]; pub(crate) per LOAD-D12f

/// One occupied registry slot — the composed per-slot struct that reconciles
/// LOAD-D8 with engine-seam SEAM-D11 (LOAD-D8 round-3 amendment; engine-seam.md
/// SEAM-D11 amended to match). Faithful `vm_s` mirror: the reuse-by-name key sits
/// beside the handle/entry it identifies (vm_local.h:119,122-123).
pub struct ModuleSlot {
    /// vm->name (vm_local.h:119): the bare module name ("jampgame"/"cgame"/"ui"),
    /// the reuse-by-name key load_module's scan compares **case-insensitively**
    /// (Raven `Q_stricmp`, vm.cpp:485-489 / game/q_shared.c:900; Rust `eq_ignore_ascii_case`,
    /// §A2 faithful-first). Stamped by load_module from its `name: &str` arg.
    /// pub(crate) (LOAD-D12f).
    pub(crate) name: String,
    /// The loaded native artifact (lib + vmMain entry). NativeDll-only today; its
    /// transport-polymorphic content for Static/Wasm is LOAD-Q9 (open).
    pub(crate) module: LoadedModule,
    /// SEAM-D11's per-slot injected engine cell — the inbound trampoline's stored
    /// syscall channel, one per slot. Concrete type owned + frozen by engine-seam
    /// SEAM-D11 (amended 2026-07-03) as
    /// `struct EngineSlot { ctx: *mut c_void, syscall: SlotSyscall }`
    /// (`engine-seam.md:547`), in this same `mp_engine_qcommon` crate — a porter
    /// names `EngineSlot` here (a same-crate type), not an opaque placeholder.
    /// Injected by `load_module` from its `system_calls`/`ctx` params (Raven's
    /// stored `vm->systemCall`, `vm.cpp:506`); the opaque `ctx` means this crate
    /// never names `mp_engine_core::Engine` (LOAD-Q11 dissolved, LOAD-D8 2026-07-03).
    pub(crate) engine: EngineSlot,
}

pub struct ModuleRegistry {
    slots: [Option<ModuleSlot>; MAX_VM],   // MAX_VM = 3 (vm.cpp:28-29)
}

impl ModuleRegistry {
    /// VM_Create slot semantics (vm.cpp:471 region). First the bad-parms guard
    /// (vm.cpp:480-482): `if name.is_empty()` →
    /// `com_error(ErrorLevel::ERR_FATAL, "VM_Create: bad parms")` (LOAD-D11, amended
    /// 2026-07-03 — the **sole** reachable disjunct of Raven's
    /// `!module || !module[0] || !systemCalls` is `!module[0]` (empty name); `!module`
    /// (a null pointer) is vacuous for a `&str`, and `!systemCalls`'s dual is the
    /// non-nullable `system_calls: SlotSyscall` (a Rust fn pointer, never null), so
    /// both drop as structurally unreachable. `syscall: RawSyscall` is a *different*
    /// Raven parameter — Sys_LoadDll's trampoline, not VM_Create's `systemCalls` — and
    /// is **not** guarded, matching Raven (vm.cpp:480-482 does not test it)). Then reuse a live slot
    /// whose stored name matches — scan
    /// `slots[i].as_ref().map(|s| s.name.eq_ignore_ascii_case(name))` (Raven reuse is
    /// `Q_stricmp`, case-insensitive — vm.cpp:486 / game/q_shared.c:900; `eq_ignore_ascii_case`
    /// reproduces its ASCII a–z fold exactly, §A2 faithful-first) against each
    /// occupied slot's `ModuleSlot.name` (vm.cpp:485-489, returned
    /// as-is, NO reload → `Some(slot_id)`), else the first free slot
    /// (`slots[i].is_none()`, vm.cpp:494), else `com_error(ErrorLevel::ERR_FATAL, …)`
    /// when all MAX_VM slots are full (vm.cpp:499-500). A fresh slot runs
    /// `sys_load_dll(policy, name, syscall)` — handing the module the raw C-shim
    /// syscall trampoline `syscall` (Raven's `VM_DllSyscall` passed to `Sys_LoadDll`,
    /// vm.cpp:518) via `dllEntry` — and on a hit wraps the returned `LoadedModule`
    /// into a `ModuleSlot { name, module, engine }`, where `engine` is the
    /// **injected** `EngineSlot { ctx, syscall: system_calls }` constructed here
    /// from `load_module`'s own `ctx`/`system_calls` params (Raven stores its
    /// `systemCalls` arg in `vm->systemCall`, vm.cpp:506), then returns
    /// `Some(slot_id)`.
    ///
    /// **The two fn-pointer params are Raven duals (LOAD-D8's 2026-07-03 amendment).**
    /// `syscall: RawSyscall` is `Sys_LoadDll`'s trampoline argument — the address
    /// handed to the module's `dllEntry` (Raven `VM_DllSyscall`, vm.cpp:518). The
    /// `system_calls: SlotSyscall` + `ctx: *mut c_void` pair is `VM_Create`'s
    /// `systemCalls` argument (vm.cpp:471-472), stored in the slot's `EngineSlot`
    /// (Raven `vm->systemCall = systemCalls`, vm.cpp:506) for the C-shim trampoline
    /// to read and forward — mirroring how `VM_DllSyscall` reads
    /// `currentVM->systemCall`. Injecting them at load is what lets
    /// `mp_engine_qcommon` avoid naming `mp_engine_core::Engine` (LOAD-Q11 dissolved;
    /// `engine-seam.md:545-570`).
    ///
    /// **Two in-`VM_Create` fatals stay internal (LOAD-D11).** The bad-parms
    /// (vm.cpp:480-482) and slot-full (vm.cpp:499-500) branches each reproduce Raven's
    /// `Com_Error(ERR_FATAL)` by calling the receiverless
    /// `mp_engine_qcommon::com_error(ErrorLevel::ERR_FATAL, …)` directly (same crate —
    /// `ErrorLevel` lives beside `com_error` in `mp_engine_qcommon`, so both resolve
    /// crate-locally from `load_module` with no cross-crate `use`: STATE-D7 fixes that
    /// crate home (state-ownership.md:1449), and lifecycle.md:662 freezes
    /// `pub type ErrorLevel = errorParm_t` over the existing per-mode
    /// `mp_qshared::errorParm_t`; exactly Raven-shaped call geometry), a diverging
    /// `-> !` panic that unwinds to the `mp_engine_core` catch (DEC-08 model). Raven puts THOSE two fatals inside
    /// `VM_Create`, so they stay inside `load_module` — both branches are fillable the
    /// moment a porter starts the body.
    ///
    /// **Not-found returns `None` (LOAD-D11, amended 2026-07-03 — resolving LOAD-Q10).**
    /// When a *fresh* slot's `sys_load_dll` returns `None` (artifact not found anywhere
    /// on the policy), `load_module` **returns `None`**, mirroring `sys_load_dll`'s own
    /// `Option` contract. This is faithful (§A2): Raven's `VM_Create` itself returns
    /// `NULL` **non-fatally** on load-not-found (its QVM fallback is out of scope,
    /// DEC-05.4), and the **caller** owns the fatal disposition, which is **non-uniform
    /// per mode** — `SV_InitGameProgs`/`CL_InitUI` `ERR_FATAL`, `CL_InitCGame`
    /// `ERR_DROP` (see `## Raven ground truth`). The `SV_InitGameProgs`-equiv boot call
    /// site reproduces its `if (!gvm) Com_Error(ERR_FATAL, …)` (`sv_game.cpp:1750-1752`)
    /// by matching this `None` and calling the receiverless `com_error` itself. No fatal
    /// is baked in here, so no per-mode divergence is pre-empted and this branch now has
    /// a fully compilable body. The remaining Slice-0 dependency is only *where* the boot
    /// call site lives (SEAM-D7 / `lifecycle.md`), not this disposition.
    pub fn load_module(&mut self, policy: &ModuleSearchPolicy, name: &str,
                       syscall: RawSyscall, system_calls: SlotSyscall,
                       ctx: *mut c_void) -> Option<SlotId>;

    /// VM_Free (vm.cpp:605-610): unload_module the slot's module, clearing it.
    /// No global currentVM/lastVM clobber (LOAD-D5).
    pub fn unload(&mut self, slot: SlotId);

    /// Native VM_Restart = drop+recreate in place (vm.cpp:398-409): unload then
    /// reload the same slot. `kind` is **caller-supplied** (LOAD-D12b — threading
    /// RestartKind through the frozen signature): `DropRecreate` for NativeDll /
    /// Static, `WasmInPlaceReset` for the wasm fast path (LOAD-D2), so the registry
    /// needs no internal transport tag to choose between them (closes LOAD-Q8).
    /// `policy`/`name`/`syscall` are still needed for `DropRecreate`'s reload.
    /// **`restart` is NOT widened with the injection params (LOAD-D8's 2026-07-03
    /// amendment; restart semantics LOAD-D2):**
    /// Raven's native `VM_Restart` saves `systemCall` off the freed `vm_t` and
    /// reuses it (vm.cpp:399-409), so the reload reuses the slot's **stored**
    /// `EngineSlot` rather than re-taking `system_calls`/`ctx` — LOAD-D12b's frozen
    /// signature stands unchanged.
    pub fn restart(&mut self, slot: SlotId, kind: RestartKind,
                   policy: &ModuleSearchPolicy, name: &str, syscall: RawSyscall);
}
```

**Transport scope of the frozen slot (LOAD-Q9, open).** `slots` holds `ModuleSlot`,
whose `module: LoadedModule` payload is **NativeDll-only** (`libloading::Library` +
`RawVmMain`) — all Slice 0 exercises (`NativeDll`, LOAD-D2 / SEAM-D7). engine-seam
(state table, line 192) makes `ModuleTransport { NativeDll | Static | Wasm }` a
**field of** `ModuleRegistry`, and DEC-05 / LOAD-D2 require the registry to also
track `Static` (no library handle) and `Wasm` (a wasmtime `Instance`, not a
`libloading::Library`) modules. **How a non-`NativeDll` module occupies a slot** —
whether `ModuleSlot.module` becomes a transport-tagged payload — and where the
`ModuleTransport` field sits relative to `slots` — is **not**
settled here (LOAD-D8 froze only `VM_Create`'s slot *reuse / allocate / overflow*
semantics over the NativeDll `ModuleSlot`) and is **not** derivable from oracle
(`Static`/`Wasm` are a jka-rust transport layer with no `vm.cpp` precedent). It is
**LOAD-Q9** — non-blocking for the NativeDll-only Slice 0. Interim forward-compat
guidance: Slice 0 references **no** `ModuleTransport` at all — the frozen NativeDll
`ModuleSlot` carries a plain `module: LoadedModule` and imports no engine-seam
transport enum, so NativeDll-only registry code compiles today **without**
`ModuleTransport` being reachable from `mp_engine_qcommon`; the polymorphic shape
change (a transport-tagged payload + the `ModuleTransport` field's placement) lands
**with** LOAD-Q9's resolution, not before.

Per-file placement (**mechanical, not architectural** — one-type-per-file /
folder-mirrors-subsystem, porting-rules; pinned only so a porter and the dry-run
land on the same paths, exactly like LOAD-D6's `native/platform` tree). The `vm/`
folder already exists mirroring oracle `qcommon/vm.cpp` (it holds `vm_s.rs`,
`opcode_t.rs`, `vm_symbol_s.rs`, `vmptr_t.rs`); the three new host-side registry
types (`SlotId`, `ModuleSlot`, `ModuleRegistry`) join it, named for the new Rust
types (not Raven identifiers):

```
crates/mp/engine/qcommon/src/vm/   // mirrors oracle qcommon/vm.cpp
  slot_id.rs          // SlotId
  module_slot.rs      // ModuleSlot (imports LoadedModule from native/platform;
                      //   EngineSlot from the sibling engine_slot.rs below)
  engine_slot.rs      // EngineSlot { ctx: *mut c_void, syscall: SlotSyscall } + the
                      //   SlotSyscall alias — struct FROZEN by engine-seam SEAM-D11
                      //   (amended 2026-07-03, engine-seam.md:545-570,547), which places
                      //   it in the mp_engine_qcommon vm.cpp-mirror subsystem
                      //   (engine-seam.md:489-490); the file follows this folder's
                      //   one-type-per-file convention (mechanical, same rationale as rest)
  module_registry.rs  // ModuleRegistry + MAX_VM
```

`EngineSlot`'s **type/shape** stays owned by engine-seam SEAM-D11 (this doc never
redefines it); its **home crate** is `mp_engine_qcommon` — the same crate as
`ModuleSlot` — and SEAM-D11 pins it to that crate's vm.cpp-mirror subsystem
(engine-seam.md:489-490), which is exactly this `vm/` folder. By the same one-type-per-file
placement this section already applies to `SlotId`/`ModuleSlot`/`ModuleRegistry`
(mechanical, porting-rules §D12), it lands in `engine_slot.rs`, so `module_slot.rs`
imports it crate-locally as `use crate::vm::engine_slot::EngineSlot` — the same-crate
path the composed slot's `engine: EngineSlot` field names. (engine-seam.md
does not itself pin the file; the placement is derived here only from its settled
subsystem + the shared mechanical convention, so it settles nothing new.) Both the
`EngineSlot` *type name* and its **fields** now resolve crate-locally: after the
2026-07-03 injection amendment the fields are an opaque `ctx: *mut c_void` and a
`SlotSyscall` fn pointer (both defined in `mp_engine_qcommon`), so there is no
longer a cross-crate `*mut Engine` field target — **LOAD-Q11 is dissolved**
(LOAD-D8's 2026-07-03 amendment; `engine-seam.md:545-570`), and
`engine_slot.rs`/`module_slot.rs` compile with no uphill edge.

**`EngineSlot`'s cross-module visibility (derived, no new decision — LOAD-D12k).**
The frozen `EngineSlot` block (engine-seam SEAM-D11, `engine-seam.md:547`, restated
verbatim in this doc) spells `struct EngineSlot { ctx: *mut c_void, syscall:
SlotSyscall }` with **no** visibility modifier — under Rust's default privacy it and
its fields would be reachable only inside `engine_slot.rs`. But this doc's own
placement (above) requires it across the same-crate one-type-per-file split:
`module_slot.rs` does `use crate::vm::engine_slot::EngineSlot` and embeds it as the
`pub(crate) engine: EngineSlot` field (LOAD-D12f), and that slot's same-crate C-shim
syscall trampoline (engine-seam SEAM-D11) reads its `ctx`/`syscall`. `EngineSlot`'s
**struct-level visibility and both fields therefore take `pub(crate)`** — the exact
LOAD-D12f mechanical convention already pinned for `ModuleSlot`/`SlotId`/`LoadedModule`
("reachable across each crate's one-type-per-file split, no wider; ordinary Rust
idiom, porting-rules §D12"). This is **forced** by the same-crate cross-module use
this doc mandates, not chosen — exactly one visibility (`pub(crate)`) satisfies it,
so it is a mechanical derivation identical to the file-placement derivation just
above (this doc never redefines the engine-seam-owned *type/shape*; it only spells
the visibility its own composition requires, derived from the shared convention —
settling nothing new). `pub(crate)` here scopes to `mp_engine_qcommon`, the crate
SEAM-D11 already homes `EngineSlot` in.

`const MAX_VM: usize = 3` (`vm.cpp:28-29`) lives in `module_registry.rs` beside
`ModuleRegistry`, mirroring oracle's `#define MAX_VM 3` sitting immediately beside
`vmTable[MAX_VM]`/`VM_Create` in `vm.cpp` — mechanical placement, same rationale
as the file mapping above, not a design choice.

`SlotId`'s wrapped `u32` is constructed by `ModuleRegistry` in the sibling
`module_registry.rs`, so it takes `pub(crate)` visibility (LOAD-D12f) — ordinary
Rust idiom for the one-type-per-file split, not a design point (porting-rules
§D12), pinned only so porters spell it uniformly. `ModuleSlot`'s fields
(`name`/`module`/`engine`) take the same `pub(crate)` treatment, constructed by
`ModuleRegistry` in `module_registry.rs`.

### Host-side wasm pointer shape (LOAD-D3)

A guest linear-memory offset is a `u32`, **never** a host address. Accessors are
bound to the module `Memory` and **re-resolve per access** (never cache a base —
`memory.grow` invalidates it) with **explicit bounds checks** (wasm memories
aren't power-of-2, so `VM_ArgPtr`'s `& dataMask` mask, `vm.cpp:652`, is replaced
by a range check; the `dataBase + offset` translation, `vm.cpp:649`, is the
precedent).

```rust
// crates/mp/engine/wasm-host (package mp_engine_wasm_host) — host-side wasm types
// + the wasmtime dep, isolated here (LOAD-D10; workspace-architecture.md).

/// A guest linear-memory offset interpreted against a module Memory (LOAD-D3).
#[repr(transparent)]
pub struct WasmPtr<T>(pub u32, core::marker::PhantomData<T>);

/// Bounds-checked, per-access accessors bound to one module's linear memory.
/// The `Memory` handle (wasmtime or equivalent) is re-read every call; no base
/// is cached. `None` = out of bounds (the sandbox fence).
pub trait ModuleMemory {
    fn read<T: bytemuck::Pod>(&self, p: WasmPtr<T>) -> Option<T>;
    fn write<T: bytemuck::Pod>(&self, p: WasmPtr<T>, v: T) -> Option<()>;
    fn bytes(&self, p: WasmPtr<u8>, len: u32) -> Option<&[u8]>;      // re-resolved
    fn bytes_mut(&mut self, p: WasmPtr<u8>, len: u32) -> Option<&mut [u8]>;
}
```

**Crate home (LOAD-D10, resolves LOAD-Q5).** `WasmPtr<T>` and `ModuleMemory` live
in the new crate `crates/mp/engine/wasm-host` (package `mp_engine_wasm_host`),
which isolates the host-side wasm types **and** the `wasmtime` dependency;
`mp_engine_qcommon`'s `ModuleTransport::Wasm` arm is feature-gated onto it, so
native-only builds carry no wasm toolchain. `docs/workspace-architecture.md`
already carries this crate (tree entry + engine paragraph, tagged "LOAD-Q5
resolution, 2026-07-02") — cited, not restated. They are host-side (bound to a
wasmtime `Memory`, so native), so they cannot live in the
`#[cfg(target_arch = "wasm32")]` guest arm of `abi-transport::entrypoints`
(LOAD-D4). Only the **topology** is settled here; the crate's internals are
designed later, before the first wasm slice (post native parity, DEC-05.5) — this
fork blocks no native slice.

Trap signatures and wire words stay **identical** across all three transports
(SEAM-D4); only the `Wasm` dispatcher arm interprets a pointer word as a
`WasmPtr<T>` and routes it through `ModuleMemory`. Which trap words are pointers
is `docs/abi-traps.md` + engine-seam SEAM-D4, not this doc.

### Per-target entrypoint modules (LOAD-D4)

The entrypoint scaffolding in `abi-transport` is `cfg`-gated by target so that
native-only code (libloading, OS handle types) is **structurally** confined to
the native arm; the wasm32 CI compile-gate (Verification strategy) then enforces
that nothing native leaks into a module built for the sandbox.

```rust
// crates/abi-transport/src/entrypoints.rs
#[cfg(not(target_arch = "wasm32"))]
pub mod native { /* extern "C-unwind" dllEntry/vmMain/GetGameAPI (STATE-D3);
                    resolved via the native/platform loader; OnceLock<CEngine> */ }

#[cfg(target_arch = "wasm32")]
pub mod wasm   { /* wasm exports vmMain/…; an IMPORTED host `syscall` (a wasm
                    function pointer can't cross via dllEntry — dossier §7);
                    WasmPtr<T> marshalling via ModuleMemory */ }
```

`crates/jampgame | cgame | ui | jagame` stay **single crates**, each wired to its
logic crate and compiled per `--target` (x86_64/i686 cdylib for native,
wasm32 for the sandbox). No separate wasm wrapper crates. The physical shell
crate that carries each module's live exports + `ENGINE` + inbound `Dispatch`
match is SEAM-D9's per-module-shell pattern; its identity is settled upstream by
SEAM-D10 (`crates/jampgame`, resolving SEAM-Q8) — this doc only fixes that
whichever crate it is, it is compiled per `--target` (its live exports live in that
shell's own `lib.rs`, not in an `abi_transport::entrypoints` arm — see the
reconciliation amendment below).

**Amendment (2026-07-03, LOAD-D4 ↔ SEAM-D9/D10 reconciliation).** The code block
above is the pre-amendment sketch and is superseded on one point: the live
`dllEntry`/`vmMain`/`GetGameAPI` exports and the `ENGINE: OnceLock<CEngine>` do
**not** live in `abi_transport::entrypoints::native`. Engine-seam **SEAM-D9/SEAM-D10**
(and its 2026-07-03 amendment, checkpoint-3 finding 14, `engine-seam.md:574-597,
873-896`) place them in each **shell crate's `lib.rs`** (`crates/jampgame/src/lib.rs`,
…), because a shared `abi_transport::entrypoints::native`'s per-`#[no_mangle]` export
collides at cdylib link with the per-shell live exports and a shared module cannot
carry a per-module `OnceLock`/`match` (`engine-seam.md:584-597`). This doc's own Scope
already punts "which physical crate hosts each module shell (`ENGINE` + live exports)"
to SEAM-D10 (§ Scope & non-goals) and the Slice-hooks section already anchors on that
shell's `lib.rs`, so this is a reconciliation to this doc's own settled Scope, **not a
new decision**. What **survives** of LOAD-D4 is its actual invariant — one crate per
module, compiled per `--target`, native-only code (libloading/OS types) structurally
confined so the wasm32 CI compile-gate stays a compiler-checked invariant — now
realized by each **shell crate** compiling per `--target`, not by a `native`/`wasm`
module split inside `abi_transport::entrypoints`. That `native`/`wasm` cfg-module split
therefore does **not** survive as an empty/structural gate either: SEAM-D9 keeps
**only** the raw C-ABI type aliases in `entrypoints.rs` (`engine-seam.md:879-880`); the
native-vs-wasm engine selection is SEAM-D13's `cfg(target_arch = "wasm32")` on the
single `type Engine` alias in the select crate (`engine-seam.md:971-975`), and the
module-side `wasm32` `Execute<C>` backend's home is engine-seam **SEAM-Q11** (open
there) — neither lives in `abi_transport::entrypoints`.

### SP linkage surface (DEC-07, LOAD-D5)

**None of this doc's MP loading apparatus has an SP dual.** SP has **no
`VM_Create`**, so it grows **no** `load_module`, **no** per-slot `EngineSlot`
injection, and **no** C-shim syscall trampoline (the whole `ModuleRegistry` /
`ModuleSlot` machinery frozen above is MP-only): its `jagame` attach is the
**direct, statically linked `GetGameAPI` call** (DEC-07; the settled SP access
discipline is state-ownership **STATE-D12** — `GetGameAPI` fn-pointer table, no
`vmMain`/`Dispatch` routing, `state-ownership.md:1617`). Raven's SP
`SV_InitGameProgs` *does* wrap this in a fake `VM_Create("cl")` shim
(`code/server/sv_game.cpp:676-679`, `## Raven ground truth`), but that shim is a
dispatch wrapper over the same `game_library` handle, not a second load, and our
port drops it (LOAD-D5). Our SP **always** uses the console/Mac fully-static
shape — `Sys_LoadCgame` / `GetProcAddress` are never exercised (dossier §4
conclusion):

- **game** — reached through the `GetGameAPI` table factory
  (`code/game/g_public.h`); the only load-shaped entry, statically linked in our
  build. In the i686-windows drop-in *hosting* scenario (matrix row 3) it is the
  one dynamically loaded artifact.
- **cgame** — reached through the vmachine shim as **pure dispatch**
  (`VM_Call → cgvm.entryPoint`, `code/client/vmachine.cpp:12-24`); retail's
  `GetProcAddress`-on-game-DLL piggyback (`win_main.cpp:557-570`) is **not
  ported** (DEC-07). The shim survives as a dispatch layer only (engine-seam
  `cgvm.entryPoint` row).
- **ui** — plain static calls (`UI_Init(...)` linked directly); never a module.

## Decisions

**LOAD-D1 — Shared mechanism, per-mode policy.** `native/platform` owns the
loading **mechanism**: one `libloading`-based loader executing a
`ModuleSearchPolicy` value; each mode's app constructs its own policy. MP order
(Win32) = `LoadLibrary`-direct, then `basepath/fs_game`, then `cdpath/fs_game`,
**no homepath** (faithful — `win_main.cpp:855-869`, `"%sx86.dll"` `:826`); **Unix
MP omits the direct probe** (its cwd `dlopen` is `#if 0`'d, `unix_main.c:361-373`)
and searches `basepath/fs_game` then `cdpath/fs_game` only (`unix_main.c:375-396`;
naming `"%si386.so"` `:346`) — carried by the per-platform `direct_first` flag. SP
= the narrower `cwd/<debugdir>` then `cwd`
lookup (`win_main.cpp:515,524`). Naming per platform: `x86.dll` / `i386.so` /
our `.dylib` addition for macOS (documented extension of the naming table,
needed for OpenJK-style hosting since oracle has no Mac MP loader). *Because* the
search-order and naming differ per mode/platform but the dlopen/handshake
mechanics are identical, one loader + a policy value avoids Raven's per-platform
`Sys_LoadDll` duplication (dossier §3) without inventing behavior. *Rejected:*
per-platform loaders (reproduces Raven's copy-paste); a single hardcoded order
(MP and SP genuinely differ).

*Amended 2026-07-02 (escalation session).* The bare/direct `LoadLibrary` probe is
**Windows-only** ground truth, so `ModuleSearchPolicy.direct_first` is set **per
platform**: Windows `true` (`win_main.cpp:855`), Unix `false` — Unix MP performs
**no** bare-`dlopen` probe, its cwd/installdir load being `#if 0`-disabled
(*"bk010205 - do not load from installdir"*, `unix_main.c:361-373`) and its order
being `FS_BuildOSPath(fs_basepath, fs_game, …)` (`unix_main.c:379`) then
`FS_BuildOSPath(fs_cdpath, fs_game, …)` (`unix_main.c:391-396`) only. The
search-order goldens (Verification 1) encode per-platform orders; Slice 0's
native-Linux policy is the faithful Unix order.

*Amended 2026-07-02 (round-3 session).* The **SP `ModuleSearchPolicy` value is
removed** from the frozen Seam. Per LOAD-D5 + DEC-07 our SP never exercises the
loader (`jagame` via the `GetGameAPI` table factory; retail SP-DLL hosting not
ported), so the frozen SP policy described a construction site (`crates/sp/app`)
that does not exist — zero-constructor surface. Its `cwd/<debugdir>` then `cwd`
search order (`win_main.cpp:515,524`) stays documented under `## Raven ground
truth` only, with a one-line why (porting-rules §20: zero-caller surface dropped
with a note). The now-orphaned `SearchStep::CwdRelative` variant is dropped with
it (its sole constructor was that SP policy). Only the **MP** per-platform policy
(Win32 / Unix) remains a live Seam value.

**LOAD-D2 — Restart = drop+recreate; wasm adds an in-place fast path.**
`NativeDll` and `Static` restart **only** by drop+recreate — Raven's real native
path (destroy+recreate on map change, `sv_init.cpp:484,662`; native `VM_Restart`
= `VM_Free`+`VM_Create`, `vm.cpp:398-409`; cgame **and** ui destroyed+recreated
every map load via `CL_FlushMemory`, dossier §1). The QVM in-place data-segment
reset is out of scope (DEC-05.4). The `Wasm` transport **additionally** offers a
QVM-style in-place reset fast path — **user-selected** — that resets linear
memory to the post-instantiation initial data image (data segments re-applied,
memory truncated to initial size, globals reset) and re-runs module init
(`dllEntry`-analog + the init `vmMain` command), **without** re-instantiating.
Both restart paths are **required to produce identical observable module
state**; a parity test between them is part of the wasm host's acceptance
(Verification strategy). *Because* native DLLs genuinely can't reset in place
(`vm.cpp:398`) while a wasm instance's linear memory can be cheaply reset without
rebuilding the `Instance`. *Rejected:* a native in-place mode (Raven proves it's
impossible for DLLs); making wasm reset a separate observable semantics (would
break parity with drop+recreate).

**LOAD-D3 — Host-side wasm pointer shape.** A `WasmPtr<T>(u32)` newtype plus
bounds-checked accessors bound to the module `Memory` (`ModuleMemory`), checked
rather than power-of-2-masked — wasm memories aren't power-of-2, so
`VM_ArgPtr`'s `& dataMask` fence (`vm.cpp:640-654`) becomes an explicit range
check while its `dataBase + offset` translation is the precedent. Trap
signatures and wire words stay identical (SEAM-D4); only the `Wasm` arm
interprets pointer words as `WasmPtr`. Accessors **re-resolve per access, never
cache a base** (`memory.grow` safety). *Rejected:* raw host pointers across the
sandbox (defeats isolation — wasm is the first transport where a pointer must
never leave as a real address, dossier §7); a cached base (invalidated by
`memory.grow`).

**LOAD-D4 — One crate per module, per-target entrypoints.**
`crates/jampgame | cgame | ui | jagame` stay single crates wired to their logic
crates, compiled per `--target` (x86_64/i686 cdylib native, wasm32 for the
sandbox). Entrypoint scaffolding is `cfg`-gated (`abi_transport::entrypoints::
native` vs `::wasm`), structurally confining native-only code (libloading, OS
types) to the native arm; the wasm32 CI compile-gate then enforces it. *Because*
one source compiled twice keeps the module logic transport-agnostic and makes
"no native code leaked into the sandbox build" a compiler-checked invariant.
*Rejected:* separate wasm wrapper crates (doubles the shell surface for no gain);
un-gated shared entrypoints (would let libloading/OS types break the wasm32
build).
*Amendment (2026-07-03, ↔ SEAM-D9/D10).* The "`abi_transport::entrypoints::native`
vs `::wasm`" placement in this record is **retracted**. Engine-seam SEAM-D9/SEAM-D10
(2026-07-03 amendment, `engine-seam.md:574-597,873-896`) place the live
`dllEntry`/`vmMain`/`GetGameAPI` + `ENGINE: OnceLock<CEngine>` in each **shell crate's
`lib.rs`**, not in an `abi_transport::entrypoints` arm — a shared arm's `#[no_mangle]`
symbols collide at cdylib link with the per-shell live exports (`engine-seam.md:589-597`),
and `abi-transport`'s `entrypoints.rs` keeps only the raw C-ABI type aliases
(`engine-seam.md:879-880`). This doc's Scope already punts the shell's physical home
to SEAM-D10 (§ Scope & non-goals), so SEAM-D9/D10 is authoritative here — **no new
decision**. LOAD-D4's surviving core is unchanged: one crate per module, compiled per
`--target`, native-only code confined so the wasm32 CI compile-gate stays a
compiler-checked invariant — now realized per shell crate. See § Per-target entrypoint
modules for the full reconciliation (including that the `native`/`wasm` cfg-module
split does not survive as a structural gate).

**LOAD-D5 — Per-slot registry, no current-module global; SP always static.** The
engine-side module registry has **no** current-module global — per-slot state
only, dispatch explicitly parameterized (engine-seam SEAM-D1) — so Raven's
`VM_Free` `currentVM`/`lastVM` clobber (`vm.cpp:624-625`) is structurally
**unreproducible** (porting-rules §A2 permits deviating here: host-side
bookkeeping, not an ABI-crossing struct). SP is **always** the fully-static shape
(DEC-07): `jagame` via `GetGameAPI` is the only load-shaped entry; cgame's
`vmMain` is reached via the vmachine shim as pure dispatch (retail's
`GetProcAddress`-on-game-DLL piggyback, `win_main.cpp:557-570`, is **not**
ported); UI is plain static calls. jampgame slice-0 hosting is `NativeDll`-first
(SEAM-D7). *Rejected:* a `currentVM`-style global for faithfulness (it is a bug,
not a contract); porting SP's retail DLL piggyback (DEC-07 drops it).

**LOAD-D6 — Raw ABI aliases live in `native/platform`; `abi-transport`
re-exports them.** (Resolves LOAD-Q4; session 2026-07-02.) The raw ABI
fn-pointer aliases `RawSyscall`, `RawVmMain`, and `RawDllEntry` (the
`dllEntry(syscall)` handshake call target — same alias category, same
`entrypoints.rs` block) **move** from
`crates/abi-transport/src/entrypoints.rs` into `crates/native/platform` — they
are Raven-free platform-ABI vocabulary, so they belong at the base tier
(`workspace-architecture.md`'s tier −1). The two supporting aliases `AbiCommand`
and `AbiWord` (`entrypoints.rs:3-4`) **move with them**: `RawVmMain`'s signature is
written in terms of both (`entrypoints.rs:10-24`), so leaving them behind would
force the very `native/platform → abi-transport` uphill edge this decision forbids
to typecheck `RawVmMain`. The relocation is therefore all **five** aliases as one
block — reconciling this record with the five-alias code block in Seam definition;
this is **forced by the no-uphill-edge constraint, not a fresh choice**, and
`abi-transport` re-exports all five (amended 2026-07-02). `abi-transport` takes a **downhill** dep
on `native/platform` and **re-exports** them from `entrypoints.rs`, so existing
`abi-transport` consumers are unaffected and no tier inversion occurs — the feared
`native/platform → abi-transport` uphill edge never forms. The FROZEN loader
signatures (`sys_load_dll`, `LoadedModule.entry`) therefore compile inside
`native/platform` naming these aliases locally, with **no new outward edge** from
the loader; `workspace-architecture.md` gains the `abi-transport →
native/platform` edge (updated there separately). The relocation carries the
`extern "C-unwind"` convention onto these aliases per **engine-seam SEAM-D12**
(adopting STATE-D3); the live `entrypoints.rs:9-27` are still the pre-STATE-D3
plain `extern "C"`, which SEAM-D12 sweeps to `-unwind` in an explicit follow-up
slice task — this doc cross-refs that sweep, it does not assert current-repo
agreement (LOAD-D12a). "Verbatim" above is arg-list/types/arity only (see the
Alias-home note in Seam definition); this applies already-settled context, not a
new choice. *Rejected:* importing
`abi-transport` into `native/platform` (inverts the tier model); redefining the
aliases locally while leaving the originals in `abi-transport` (duplicates an ABI
type).

**LOAD-D7 — `Sys_UnpackDLL` is in scope but deferred; Slice 0 stubs it.**
(Resolves LOAD-Q2; session 2026-07-02.) The pure-server unpack pre-step
`Sys_UnpackDLL` (`win_main.cpp:762-800`, called before any `LoadLibrary` at
`win_main.cpp:849-852`) **is in scope** — it is live retail pure-server behavior
on the DEC-05 hosting path — but is **deferred** to a later MP-server slice, after
the filesystem port (slice B2) lands the `FS_ReadFile` / `FS_FileIsInPAK` /
`FS_FOpenFileWrite` / `FS_Write` calls it needs. Slice 0 is non-pure and does not
exercise it, so `sys_load_dll` **stubs** it: where the pre-step would run the
porter emits a `//TODO: Port Sys_UnpackDLL` + `// Source:` marker (porting-rules
§14 — unported deps are explicit, never a silent no-op), not a silently-swallowed
step. For an on-disk (non-pak) DLL `Sys_UnpackDLL` returns `true` without writing
(`FS_FileIsInPAK == -1`, `win_main.cpp:774-779`), so the deferral changes no
observable behavior for Slice 0's on-disk `jampgame` load. *Rejected:* dropping it
(it is real retail behavior on the promised path); porting it now (blocked on the
unported B2 filesystem calls).

**LOAD-D8 — `ModuleRegistry` slot API frozen against `VM_Create`.** (Resolves
LOAD-Q3; session 2026-07-02.) The `vmTable[MAX_VM]` replacement `ModuleRegistry`
(home crate `mp_engine_qcommon`, mirroring oracle `qcommon/vm.cpp`'s subsystem per
engine-seam.md's state table) exposes a frozen slot API following `VM_Create`'s
semantics (`vm.cpp:471` region): `load_module(&mut self, policy, name, syscall) ->
SlotId` **reuses** a live slot whose module name matches **case-insensitively**
(Raven `Q_stricmp`, `vm.cpp:486` / `game/q_shared.c:900`; the Rust faithful equal is
`eq_ignore_ascii_case`, matching Q_stricmp's ASCII a–z fold, §A2), returned as-is
with **no reload** (`vm.cpp:485-489`), else takes the **first free** slot
(`vm.cpp:494`) and
runs `sys_load_dll`, else `Com_Error(ERR_FATAL)` when all `MAX_VM = 3` slots are
full (`vm.cpp:499-500`). `unload(slot)` follows `VM_Free` (`vm.cpp:605-610`) with
no `currentVM`/`lastVM` clobber (LOAD-D5); `restart(slot, …)` is native
drop+recreate in place (`vm.cpp:398-409`), Wasm substituting the in-place reset
(LOAD-D2) behind the same call. Host-side bookkeeping, not an ABI-crossing struct,
so porting-rules §A2 permits this shape. **Freeze scope:** this covers
`VM_Create`'s slot *reuse / allocate / overflow* semantics over a NativeDll-shaped
`ModuleSlot` (its `module: LoadedModule` payload; all Slice 0 needs); the
transport-polymorphic slot **content** for `Static`/`Wasm` and the
`ModuleTransport` field engine-seam places on `ModuleRegistry` (state table, line
192) are **LOAD-Q9** (open, non-blocking for
the NativeDll-only Slice 0), not frozen here. *Rejected:* modeling name-reuse or
fatal-overflow differently (they are cited `VM_Create` behavior on the hosting
path); a current-module global (LOAD-D5).

*Amended 2026-07-02 (round-3 session — slot shape resolved, reconciling the
SEAM-D11 conflict; engine-seam.md SEAM-D11 amended to match).* Two forces met on
the slot element: (i) the reuse-by-name scan (`vm.cpp:485-489`) needs the loaded
name stored on the slot, and (ii) engine-seam SEAM-D11 needs a per-slot `*mut
Engine` trampoline cell — "one cell per module slot". The registry slot element is
therefore **one composed per-slot struct** in `mp_engine_qcommon`'s
`module_registry`:

```
ModuleSlot { name: String, module: LoadedModule, engine: EngineSlot }
```

with `ModuleRegistry.slots: [Option<ModuleSlot>; MAX_VM]`. `name` is the
`vm.cpp:485-489` reuse-by-name key (the faithful `vm_s` mirror — oracle's
`vm->name` sits in the *same* slot record as `dllHandle`/`entryPoint`,
`vm_local.h:119,122-123`); `module` is the unchanged `LoadedModule { lib, entry }`;
`engine` is the SEAM-D11 cell (its **type owned by engine-seam**, this struct only
holds the field — see the 2026-07-03 amendment below for its injected shape).
`load_module`'s reuse scan reads `slot.name`, and stamps it
from the `name: &str` arg it already receives — closing the name-field hole **with
authorization**. `LoadedModule` reverts to `{ lib, entry }` (name moved up to the
slot); the free-slot test is `slots[i].is_none()` (oracle's `!vmTable[i].name[0]`,
`vm.cpp:494`). This supersedes the earlier `LoadedModule.name` placement and
reconciles LOAD-D8 with SEAM-D11 on one shared slot type. *Rejected:* a parallel
`names: [Option<String>; MAX_VM]` (splits identity from payload); leaving the
engine cell out of the slot (SEAM-D11 requires it per-slot).

*Amended 2026-07-03 (`EngineSlot` load-time injection + `load_module` widening —
reconciling this doc with engine-seam SEAM-D11's 2026-07-03 amendment,
`engine-seam.md:545-570`; supersedes the round-3 `Cell<*mut Engine>` cell + the
retired `EngineSlotGuard`/`enter` scope-guard).* The composed slot's `engine`
field is now the **injected** shape `EngineSlot { ctx: *mut c_void, syscall:
SlotSyscall }` (`engine-seam.md:547`), with
`SlotSyscall = extern "C-unwind" fn(ctx: *mut c_void, args: *const isize) -> isize`
(`engine-seam.md:545-546`). Two consequences for the frozen `ModuleRegistry` API:
**(1)** `load_module` is **widened** to
`load_module(&mut self, policy, name, syscall: RawSyscall, system_calls:
SlotSyscall, ctx: *mut c_void) -> Option<SlotId>` — the two fn-pointer params are
Raven duals: `syscall` is `Sys_LoadDll`'s trampoline argument handed to `dllEntry`
(Raven `VM_DllSyscall`, `vm.cpp:518`), while `system_calls`+`ctx` are `VM_Create`'s
`systemCalls` (`vm.cpp:471-472`), which `load_module` stores in the slot as
`EngineSlot { ctx, syscall: system_calls }` (Raven `vm->systemCall = systemCalls`,
`vm.cpp:506`). The `Option<SlotId>` return is unchanged (LOAD-D11's 2026-07-03
amendment stands). **(2)** `restart` is **not** widened: Raven's native
`VM_Restart` saves `systemCall` off the freed `vm_t` and reuses it
(`vm.cpp:399-409`), so the reload reuses the slot's stored `EngineSlot` and
LOAD-D12b's frozen `restart` signature stands. This injection is precisely what
lets `mp_engine_qcommon` avoid naming the uphill `mp_engine_core::Engine` — it
stores what it is **handed** at load, mirroring `VM_Create`'s stored `systemCalls`
— **resolving LOAD-Q11** (Open questions → Resolved). *Rejected:* keeping the
per-call `Cell<*mut Engine>` cell + guard (rustc surfaced the forbidden uphill
edge, `engine-seam.md:555-557`); re-taking the injection on `restart` (Raven reuses
the stored `systemCall`, `vm.cpp:399-409`).

**LOAD-D9 — `SearchStep::FsPath` carries resolved paths; the policy is built at
the `mp_engine_qcommon` call site (resolve-at-construction).** (Resolves LOAD-Q6;
session 2026-07-02.) `SearchStep::FsPath` carries **resolved** values —
`{ base: PathBuf, gamedir: String }` — **not** cvar names. The
`ModuleSearchPolicy` value is built **per load by the caller in
`mp_engine_qcommon`** (at the `ModuleRegistry::load_module` call site): once the
cvar port (B1) lands the caller reads `Cvar_VariableString("fs_basepath")` /
`("fs_cdpath")` / `("fs_game")` (`win_main.cpp:858-860`) and plants the results;
Slice 0 (pre-B1) builds the same value from hardcoded / CLI paths.
`native/platform` therefore **never reaches a cvar table** (porting-rules
§B3/§B4 — no hidden globals, state threaded not reached), and `sys_load_dll`'s
FROZEN signature `(policy, name, syscall)` stands **unchanged** — no cvar-resolver
param is threaded through it. The only frozen-type change is `FsPath`'s fields
(names → resolved values), authorized by this resolution. Search-order goldens
(Verification 1) are described over the **resolved per-platform path sequences**.
*Because* resolving at the qcommon construction site keeps the tier −1 loader
cvar-free while still honoring `win_main.cpp`'s `Cvar_VariableString` reads.
*Rejected:* threading a cvar-resolver closure into `sys_load_dll` (extends the
frozen seam; leaks cvar reach into `native/platform`); keeping cvar *names* in
`FsPath` (forces `native/platform` to reach a cvar table it must not know about).

*Amended 2026-07-02 (round-3 session — cdpath skip resolved; supersedes the
earlier in-loader placement).* Raven only probes the cdpath location `if
(cdpath[0])` (`win_main.cpp:866`, `unix_main.c:391`); an empty `fs_cdpath` skips
that probe. The skip is owned by the **`mp_engine_qcommon` policy-construction
function**, which **omits any `FsPath` step whose `base` is empty** — so the
policy handed to `sys_load_dll` contains only real steps and the loader **executes
it blindly** (walks every step it is given, first hit wins). This keeps
`native/platform` cvar-semantics-free (porting-rules §B3/§B4): the loader carries
no emptiness-awareness, and the `if (cdpath[0])` condition is realized where the
cvar is read, at construction. The search-order goldens (Verification 1) are over
**constructed policies**, so the skip is golden-tested at the construction layer
(an empty-`fs_cdpath` fixture yields a one-step policy). On every reachable input
this reproduces oracle exactly: `fs_basepath` is the resolved install path (never
empty, attempted unconditionally per `win_main.cpp:862-863`), only `fs_cdpath` is
empty-able. *Rejected:* placing the skip inside `sys_load_dll` (would leak
cvar-emptiness semantics into the cvar-free loader — the earlier round-2 placement,
now superseded).

**LOAD-D10 — Host-side wasm types live in `crates/mp/engine/wasm-host`.**
(Resolves LOAD-Q5; session 2026-07-02.) `WasmPtr<T>` and `ModuleMemory` (LOAD-D3)
live in a new crate `crates/mp/engine/wasm-host` (package `mp_engine_wasm_host`),
which isolates both the host-side wasm types **and** the `wasmtime` dependency;
`mp_engine_qcommon`'s `ModuleTransport::Wasm` arm is **feature-gated** onto it, so
native-only builds carry no wasm toolchain. `docs/workspace-architecture.md` is
**already updated** (crate tree entry + engine-paragraph note, tagged "LOAD-Q5
resolution, 2026-07-02") — cited, no contradiction expected. Only the **topology**
is settled; the crate's internals are designed later, before the first wasm slice
(post native parity, DEC-05.5). *Because* the host-side wasm types are native (not
`wasm32` guest types, LOAD-D4) yet must not drag `wasmtime` into every native
build — a dedicated feature-gated crate isolates the dep. *Rejected:* putting them
in `crates/native/platform` (would pull `wasmtime` into the base platform tier);
the `#[cfg(target_arch = "wasm32")]` guest arm of `abi-transport::entrypoints`
(they bind a host `Memory`, not a guest type).

**LOAD-D11 — `load_module`'s `ERR_FATAL` is a direct `com_error` panic; not-found
returns `None` (`-> Option<SlotId>`, per the 2026-07-03 amendment below).**
(Resolves LOAD-Q7; session 2026-07-02.) Following the STATE-D7
split shape (resolving STATE-Q4) settled and **FROZEN** in `state-ownership.md`
(now in this doc's Standing context; the earlier receiver-ful `lifecycle.md`
shape is superseded), `ModuleRegistry::load_module` reproduces Raven's slot-full
`Com_Error(ERR_FATAL)` (`vm.cpp:499-500`) by calling the **receiverless**
`mp_engine_qcommon::com_error` **directly** — its exact frozen signature is
`pub fn com_error(level: ErrorLevel, msg: String) -> !` (state-ownership STATE-D7,
`state-ownership.md:719`), invoked with the concrete fatal variant
`ErrorLevel::ERR_FATAL` — `ErrorLevel` aliases the ported `errorParm_t` enum whose
first member is `ERR_FATAL` (`oracle/codemp/game/q_shared.h:451-457`;
state-ownership STATE-D7 / lifecycle LIFE-D3 name the alias) — and a formatted `String`.
`ModuleRegistry` lives in the same crate, so this is exactly Raven-shaped call
geometry — a diverging `-> !` panic that unwinds to the `mp_engine_core` catch
(DEC-08 model). The porter therefore fills the slot-full branch with a real
`com_error(ErrorLevel::ERR_FATAL, format!(…))` call reachable from the standing set,
**not** a `//TODO: Port` marker.
`load_module` keeps its **infallible `-> SlotId`** return (no `Result` variant);
the diverging-panic path is noted at the signature. *Because* the slot-full case
is a real cited `VM_Create` fatal, and a direct same-crate `com_error` call
reproduces Raven's geometry without widening the frozen return type. *Rejected:* a
`Result` fatal variant on `load_module` (widens the frozen infallible return);
routing the fatal through a threaded receiver (STATE-D7 settled `com_error`
receiverless).

*Amended 2026-07-02 (dry-run hole-close, no new fork).* `VM_Create` has a *second*
earlier `ERR_FATAL` — the bad-parms guard `if ( !module || !module[0] ||
!systemCalls ) Com_Error( ERR_FATAL, "VM_Create: bad parms" )` (`vm.cpp:480-482`)
— reachable through `load_module`'s frozen surface (an empty `name: &str`, a null
`RawSyscall`). It gets the **identical** disposition this decision already settled
for the slot-full fatal: `load_module` reproduces it as a first-statement guard
`if name.is_empty() || syscall.is_null() { com_error(ErrorLevel::ERR_FATAL,
"VM_Create: bad parms".into()) }`. This is faithful-first (§A2) applied to the sibling fatal in
the same function under LOAD-D11's already-chosen mechanism — **not** a new
choice, so neither `//TODO: Port` nor a skip. Of Raven's three disjuncts, `!module`
(a null `const char*`) is vacuous for a Rust `&str` and is dropped as
structurally-unreachable (cf. LOAD-D5's dropped `currentVM` clobber); `!module[0]`
(empty) and `!systemCalls` (null syscall) are the reachable, reproduced ones.
`load_module` keeps its infallible `-> SlotId`.

*Amended 2026-07-03 (LOAD-Q10 resolution — supersedes the "infallible `-> SlotId`"
return in the original record and its bad-parms amendment above).* `load_module`
**returns `Option<SlotId>`**: a fresh-slot `sys_load_dll(...) -> None` (artifact not
found on any policy step) yields **`None`**, mirroring `sys_load_dll`'s own `Option`
contract. This is faithful (§A2), not a widened error channel invented for
convenience: Raven's `VM_Create` **itself returns `NULL` non-fatally** on
load-not-found (the adjacent QVM fallback is out of scope, DEC-05.4), and **the
caller owns the fatal disposition** — which is **non-uniform per mode**
(`SV_InitGameProgs`/`CL_InitUI` `ERR_FATAL`, `CL_InitCGame` `ERR_DROP`; see
`## Raven ground truth`). The `SV_InitGameProgs`-equiv boot call site reproduces
`gvm = VM_Create(...); if (!gvm) Com_Error(ERR_FATAL, "VM_Create on game failed")`
(`sv_game.cpp:1750-1752`) by matching `None` and calling the receiverless `com_error`
itself. **The two in-`VM_Create` `ERR_FATAL`s stay inside `load_module`** (bad-parms
`vm.cpp:480-482`, slot-full `vm.cpp:499-500`) — Raven puts THOSE inside `VM_Create`,
so LOAD-D11's original mechanism (direct receiverless `com_error`) is unchanged for
them; only the caller-side not-found disposition moves out via the `Option`. This
closes **LOAD-Q10**: the earlier "genuine fork = whether to amend FROZEN LOAD-D11"
is settled by amending it (return widened to `Option<SlotId>`), and the earlier
claim that the not-found case was "not derivable from oracle ground truth" is
**withdrawn** (oracle derives it at the caller, `sv_game.cpp:1750-1752`). *Because* a
per-mode fatal split cannot be reproduced by a single internal `ERR_FATAL`, so the
`None` must surface to the caller — exactly Raven's own contract. *Rejected:* a
uniform internal `ERR_FATAL` on not-found (diverges from cgame's `ERR_DROP`); a
`Result<SlotId, E>` (the disposition needs no error payload — `None` is Raven's
`NULL`, and the caller re-derives the level per mode).

*Amended 2026-07-03 (round-6 stamping — bad-parms guard corrected; closes the
round-5 escalation; supersedes the `|| syscall.is_null()` disjunct in the
2026-07-02 bad-parms amendment above).* The 2026-07-02 amendment reproduced Raven's
`if ( !module || !module[0] || !systemCalls )` guard (`vm.cpp:480-482`) as
`if name.is_empty() || syscall.is_null()`, mapping `!systemCalls` to
`syscall.is_null()`. That mapping is **wrong** and is dropped: `syscall: RawSyscall`
is `Sys_LoadDll`'s trampoline argument (Raven's `VM_DllSyscall`, `vm.cpp:518`), a
**different** Raven parameter than `VM_Create`'s `systemCalls`. `systemCalls`' actual
Rust dual is the settled `system_calls: SlotSyscall` (LOAD-D8's 2026-07-03 injection
amendment), a **non-nullable** Rust fn pointer — exactly the class of Raven's
`!module` disjunct that this decision already drops for a `&str` (both are
null-pointer tests with no reachable Rust dual). The guard is therefore
**`if name.is_empty()` only** — the sole reachable disjunct is `!module[0]` (empty
name). `syscall: RawSyscall` is **not** guarded, matching Raven, which does not test
its `Sys_LoadDll` trampoline argument either (`vm.cpp:480-482` guards only `module`
and `systemCalls`). Observable behavior is identical: the "bad parms" fatal never
fires in legitimate use, and the removed `syscall.is_null()` branch tested a
parameter Raven never guards. This is faithful-first (§A2) — a structurally-unreachable
disjunct dropped like `!module` and LOAD-D5's `currentVM` clobber — **not** a new
choice. *Rejected:* keeping `syscall.is_null()` relabeled as defensive-Rust
(speculative divergence, §A2); an `Option<SlotSyscall>` ceremony to make
`systemCalls` nullable (invents a null state Raven's non-null fn pointer never has).

**LOAD-D12 — Mechanical closes (no forks).** (Session 2026-07-02; **(f)** added
round-3 2026-07-02.) Mechanical hole-closes, none a new design choice: **(a)** the relocated `RawSyscall` /
`RawVmMain` / `RawDllEntry` aliases (LOAD-D6) are `extern "C-unwind"` per
**engine-seam SEAM-D12**; the repo's current plain-`extern "C"` aliases
(`entrypoints.rs:9-27`) are swept to `-unwind` in the already-tracked SEAM-D12
**follow-up slice task** — this doc cross-refs that sweep rather than asserting
current-repo agreement. **(b)** `RestartKind` (LOAD-D2) is threaded through the
frozen `ModuleRegistry::restart` as a **caller-supplied** `kind: RestartKind`
parameter (`DropRecreate` for NativeDll/Static, `WasmInPlaceReset` for the wasm
fast path), so the registry needs no internal transport tag to dispatch — this
closes LOAD-Q8. **(c)** `const MAX_VM: usize = 3` (`vm.cpp:28-29`) lives in
`module_registry.rs` alongside `ModuleRegistry`, mirroring oracle's `#define
MAX_VM 3` beside `vmTable[MAX_VM]`. **(d)** Cargo wiring:
`crates/native/platform/Cargo.toml` gains a `libloading` dependency used only by
`module_loader/loader.rs`, and `crates/abi-transport/Cargo.toml` gains the
`native_platform` path dependency (LOAD-D6's downhill re-export edge). **(e)** the
five relocated aliases (LOAD-D6) — `AbiCommand`, `AbiWord`, `RawSyscall`,
`RawDllEntry`, `RawVmMain` — land in `crates/native/platform/src/entrypoints.rs`, a
sibling of `module_loader/` (they are ABI vocabulary, not loader types, so they sit
outside that tree), mirroring the `abi-transport` `src/entrypoints.rs` they move
from; `abi-transport` re-exports all five from its own `entrypoints.rs`. **(f)**
(round-3) field visibility is pinned `pub(crate)`: `LoadedModule.lib`/`.entry`
(constructed by `loader.rs` within `native/platform`) and `SlotId`'s tuple field
plus `ModuleSlot`'s `name`/`module`/`engine` (constructed by `module_registry.rs`
within `mp_engine_qcommon`) — reachable across each crate's one-type-per-file split,
no wider; ordinary Rust idiom (porting-rules §D12), pinned only so porters spell it
uniformly. *Rejected:* n/a — mechanical, no alternatives.

*Amended 2026-07-03 (round-5 mechanical sweep, skeleton-seed findings — no forks).*
**(g)** `SlotSyscall` (the injected syscall fn-pointer alias, `engine-seam.md:545-546`)
and `c_void` are named crate-locally in `mp_engine_qcommon` by the widened
`load_module` and the `EngineSlot` field (LOAD-D8 2026-07-03 amendment); the C-shim
syscall trampoline whose address is handed as `syscall: RawSyscall` is engine-seam
SEAM-D11's committed `vm/game_syscall_trampoline.c` (`cc` build dep confined to
`mp_engine_qcommon`, `engine-seam.md:527-543`), cross-referenced, not owned here.
**(h)** Sibling-doc status annotations are current: `lifecycle.md` and
`state-ownership.md` **exist on disk** (DRAFT / FROZEN sections as cited) — no stale
"(pending)" annotation remains (the sole surviving "pending" is the still-open
LOAD-Q1 macOS-suffix golden). **(i)** The cross-doc deferral pointers for the
registry attachment (STATE-D10 / LIFE-Q5, `Engine.common.modules`) and the
`SV_InitGameProgs`-equiv call site (SEAM-D7 + `lifecycle.md:69-70` mutual deferral,
LOAD-Q12) were re-verified precise under the full sibling reading set; text
unchanged. *Rejected:* n/a — mechanical.

*Amended 2026-07-03 (round-6 dry-run hole-close — derived visibility; no forks).*
**(k)** `EngineSlot`'s
struct-level and `ctx`/`syscall` field visibility is **derived** `pub(crate)` — forced
by the same-crate `module_slot.rs` `use` + C-shim trampoline read this doc mandates,
under the identical LOAD-D12f one-type-per-file convention (§ Per-file placement,
"EngineSlot's cross-module visibility"). This spells only the visibility the
composition requires; the engine-seam-owned SEAM-D11 *type/shape* is unchanged.
*Rejected:* n/a — mechanical.

The bad-parms fatal (the second `VM_Create` `ERR_FATAL`, `vm.cpp:480-482`) is covered by LOAD-D11's amendment above
(added to `## Raven ground truth` and LOAD-D11's enumeration, reproduced via the
receiverless `com_error`), not restated here.

### DEC-05 drop-in parity matrix (both directions)

| # | Host | Module | Platform / build target | Transport | Notes |
|---|---|---|---|---|---|
| 1 | Rust engine | Rust module | any native (x86_64 / i686 / arm64) cdylib | NativeDll | core scenario (DEC-05.1); `native/platform` loader, LOAD-D1 |
| 1s | Rust engine | Rust module | same crate, statically linked | Static | our-engine hosting; no load step (engine-seam SEAM-D1) |
| 1w | Rust wasmtime host | Rust module | wasm32 | Wasm | DEC-05.5; lands **after** native parity |
| 2a | retail `jamp` 1.01 | Rust module | **i686-pc-windows** cdylib (`…x86.dll`) | NativeDll | drop-in (DEC-05.2); seam structs oracle-exact |
| 2b | OpenJK native | Rust module | native cdylib per platform | NativeDll (+ `GetModuleAPI`) | DEC-05.2; game-private layout per-host (NOTES.md:65-68); `GetModuleAPI` contract SEAM-Q7 |
| 3 | Rust engine | real / mod DLL (JA+, MBII, …) | **i686-pc-windows engine only** | NativeDll (hosting) | DEC-05.3; 32-bit PE; raw inbound trampoline SEAM-D11 |
| gate | `cargo build` | Rust module crates | wasm32-unknown-unknown | — | standing CI compile-gate from day one (DEC-05.5) |

## Verification strategy

Per DEC-09, native track (porting-rules §E — green at every commit, one
file/function per commit):

1. **Loader TU tests** (DEC-09.1 pattern): search-order goldens — construct
   fixture directories that plant a fake artifact at each candidate location and
   assert `sys_load_dll` walks the MP **Win32** order (`LoadLibrary`-direct →
   basepath → cdpath, no homepath) and the MP **Unix** order (basepath → cdpath,
   **no direct probe**) exactly, first-hit-wins, matching `win_main.cpp:855-869` /
   `unix_main.c:361-396`. (There is **no SP-order golden**: our SP constructs no
   policy — LOAD-D1 round-3 — so its `cwd/<debugdir>` → `cwd` order,
   `win_main.cpp:515,524`, is ground truth only.) Goldens are over **constructed
   policies** (LOAD-D9 round-3) — `FsPath` steps carry already-resolved
   `base`/`gamedir` and any empty-`fs_cdpath` step is omitted at construction, so
   the empty-cdpath skip is golden-tested at the construction layer and
   `sys_load_dll` never reads a cvar. Naming-table goldens assert `"jampgame" →
   jampgamex86.dll` (Win32, `suffix: Some("x86.dll")`) / `jampgamei386.so` (Unix,
   `Some("i386.so")`) per platform; the macOS naming golden is **pending LOAD-Q1**
   (its exact suffix is unresolved, so no macOS `ModuleNaming` is constructed yet —
   `suffix` would be `None`, LOAD-Q1), and Slice 0's win/linux targets do not
   exercise it.
2. **Live-peer** (DEC-09.2, DEC-05.2): our module cdylib (`…x86.dll`,
   i686-pc-windows) loaded by an **unmodified OpenJK/retail** engine — the
   `dllEntry`+`vmMain` handshake round-trip against a real host (matrix rows
   2a/2b); and the reverse — **our engine hosting an OpenJK-built module DLL**
   (matrix row 3, i686-pc-windows).
3. **Wasm restart-equivalence parity test** (LOAD-D2): drive a module through
   `RestartKind::WasmInPlaceReset` and `RestartKind::DropRecreate` from the same
   pre-restart state and assert **byte-identical** observable module state after
   — the acceptance gate for the in-place fast path. Lands with the wasmtime host
   (after native parity, DEC-05.5).
4. **wasm32 compile-gate** (DEC-05.5), standing CI from day one: the four module
   crates build for `wasm32-unknown-unknown`; a native-only symbol (libloading,
   OS handle) leaking past the `cfg` gate (LOAD-D4) fails the build.

## Slice hooks

- **Slice 0 (MP dedicated boot)** needs the `native/platform` loader
  (`sys_load_dll`, `ModuleSearchPolicy`) + the **jampgame policy only** frozen
  here (LOAD-D1) — SEAM-D7 boots `jampgame` on `NativeDll` through it. Restart is
  `RestartKind::DropRecreate` (LOAD-D2). No wasm, no cgame/ui. The three forks
  that previously gated the compilable skeleton are now **resolved**, so Slice 0
  is unblocked: its `sys_load_dll` **target artifact** — the `crates/jampgame`
  thin cdylib shell whose `dllEntry`/`vmMain` the loader resolves — is settled by
  SEAM-D10 (resolving SEAM-Q8 — cited, not restated; the shell's `lib.rs` holds
  only `ENGINE`/exports/`Dispatch` match, no loader call); the `RawSyscall` /
  `LoadedModule.entry: RawVmMain` aliases are named locally in
  `crates/native/platform` (LOAD-D6, resolving LOAD-Q4); and the engine-side
  `SV_InitGameProgs`-equiv boot — where the `sys_load_dll` call site lives —
  registers the loaded `jampgame` slot through the frozen
  `ModuleRegistry::load_module(policy, name, syscall, system_calls, ctx) ->
  Option<SlotId>` (LOAD-D8, resolving LOAD-Q3; return widened by LOAD-D11's
  2026-07-03 amendment, params widened by LOAD-D8's 2026-07-03 injection amendment).
  The slot's `engine: EngineSlot` cell — previously the LOAD-Q11 compilability
  blocker — is now the **injected** `EngineSlot { ctx, syscall: system_calls }`
  built from `load_module`'s own params, so `mp_engine_qcommon` names no uphill
  `Engine` and `engine_slot.rs`/`module_slot.rs` compile standalone (**LOAD-Q11
  resolved**, LOAD-D8's 2026-07-03 amendment). The
  pure-server `Sys_UnpackDLL` pre-step is stubbed with a `//TODO: Port` marker
  (LOAD-D7) — Slice 0 is non-pure and does not exercise it. **LOAD-Q10 is now
  resolved** (LOAD-D11 amended 2026-07-03): `load_module` returns
  `Option<SlotId>`, so its fresh-slot *not-found* branch has a compilable body
  (`None`) and the missing-`jampgame` path is expressible — the boot call site
  reproduces Raven's `if (!gvm) Com_Error(ERR_FATAL, …)` (`sv_game.cpp:1750-1752`)
  by matching that `None`. One caveat remains: the `SV_InitGameProgs`-equiv load
  call site's **trigger** is settled — map spawn (`SV_SpawnServer → SV_InitGameProgs`,
  post-Slice-0, `lifecycle.md:82,222`), not engine boot (Slice-0 `Com_Init` only
  default-constructs the empty registry at step-30 `VM_Init`) — but its own
  **crate/signature** (and *where* that `None`-match lives) is pinned by **neither**
  this doc nor lifecycle.md (they mutually defer, `lifecycle.md:69-70`): **LOAD-Q12**,
  **owned outside this doc**. The loader surface builds standalone; end-to-end wiring
  waits on it (Referenced-but-owned-elsewhere note). The `FsPath` cvar-resolution path the
  jampgame policy relies on is **settled (LOAD-D9)** — the `mp_engine_qcommon`
  caller plants resolved values (Slice 0 uses hardcoded/CLI paths pre-B1), so it no
  longer blocks Slice 0.
- **Client slices** add cgame/ui loading (same MP policy, same drop+recreate
  cadence, dossier §1) and their registry slots.
- **SP slice** wires the `GetGameAPI` table + vmachine shim dispatch — no loader
  path exercised (DEC-07, LOAD-D5).
- **Wasm host slice** (post native-parity, DEC-05.5): the `wasm` entrypoint arm,
  `ModuleMemory`/`WasmPtr` in `crates/mp/engine/wasm-host` (LOAD-D10), and the
  restart-equivalence parity test driving the caller-supplied
  `RestartKind::WasmInPlaceReset` vs `DropRecreate` (LOAD-D12b).

## Open questions

**Open — returns to a design session:**

- **LOAD-Q1 — Exact macOS artifact filename.** LOAD-D1 settles the macOS naming
  entry as our `.dylib` extension, but oracle has **no** Mac MP loader (dossier
  §3), so the precise base string — whether the module name carries an arch infix
  (e.g. OpenJK's `…arm64.dylib` / `…x86_64.dylib`) or a bare `…​.dylib` — cannot
  be derived from oracle ground truth and must match whatever OpenJK-style host
  we intend to interoperate with (a DEC-05.2 interop detail). `ModuleNaming.suffix`
  was **widened to `Option<&'static str>`** (mechanical, 2026-07-03) so the macOS
  arm is representable as **`None`** — "suffix not yet resolved" — rather than
  forcing a placeholder literal into a mandatory field. Until LOAD-Q1 resolves, no
  macOS `ModuleNaming` value is constructed for Slice 0 at all: a `//TODO: Port
  <macOS module suffix>` + `// Source:` LOAD-Q1 marker stands in that macOS-only
  construction arm (porting-rules §14), and Slice 0's i686-windows / native-Linux
  targets never compile it. Slice 0 constructs only the Win32 (`Some("x86.dll")`)
  and Unix (`Some("i386.so")`) naming values; when the macOS host is wired,
  LOAD-Q1's resolution supplies the real literal (`Some("…")`). The Win32/Unix
  entries and the whole native-parity path are unaffected by the widening. **Owner:**
  resolved by verifying against the OpenJK-style host the module targets, when the
  macOS host is wired; not needed for Slice 0 (i686-windows / native-Linux only).

- **LOAD-Q9 — Transport-polymorphic slot content for `Static` / `Wasm` modules.**
  The frozen `ModuleRegistry.slots: [Option<ModuleSlot>; MAX_VM]` (LOAD-D8 round-3)
  holds a `ModuleSlot` whose `module: LoadedModule` payload is **NativeDll-only**
  (`libloading::Library` + `RawVmMain`) — all Slice 0 exercises. engine-seam (state
  table, line 192) makes `ModuleTransport { NativeDll | Static | Wasm }` a **field
  of** `ModuleRegistry`, and DEC-05 / LOAD-D2 require the registry to also hold
  `Static` (no library handle) and `Wasm` (a wasmtime `Instance`, not a
  `libloading::Library`) modules — but neither this doc nor any cited sibling
  settles **how such a module occupies a slot** (e.g. `ModuleSlot.module` becoming a
  transport-tagged payload, or a slot pairing a `ModuleTransport` tag with a
  transport-specific payload) or **where the `ModuleTransport` field sits relative
  to `slots`**. Not derivable from oracle (`Static`/`Wasm` transports have no
  `vm.cpp` precedent — they are a jka-rust layer). **Non-blocking for Slice 0**
  (NativeDll only — Slice 0 imports no `ModuleTransport`, so no interim reachability
  of that enum from `mp_engine_qcommon` is required; NativeDll-only registry code
  compiles today without it). **Owner:** design session, before the first `Static`
  (client / SP slices) or `Wasm` (post-native-parity, DEC-05.5) registry slot lands.

- **LOAD-Q12 — The `SV_InitGameProgs`-equiv load call site's crate/signature
  (mutual-deferral ownership).** *When* the game module loads is settled by
  lifecycle.md — map spawn (`SV_SpawnServer → SV_InitGameProgs`, post-Slice-0), not
  engine boot (`lifecycle.md:82,187-215,222`; Slice-0 `Com_Init` only builds the empty
  registry at step-30 `VM_Init`). What is **unsettled** is the our-engine
  `SV_InitGameProgs`-equiv function's own **crate + exact signature** (and where the
  `load_module -> Option<SlotId>` `None`-match reproducing `if (!gvm)
  Com_Error(ERR_FATAL, …)`, `sv_game.cpp:1750-1752`, physically lives): lifecycle.md
  **punts `SV_InitGameProgs` module-load mechanics back to this doc**
  (`lifecycle.md:69-70`) while this doc's Scope punts the call site to lifecycle.md +
  engine-seam **SEAM-D7** — so neither doc owns it. Not a Slice-0 blocker: SEAM-D7's
  Slice-0 model hosts our module **inside a real/OpenJK engine** (which supplies the
  boot sequencing), so the *our-engine* boot function is a later-slice deliverable;
  the loader/registry surface builds standalone (Verification 1). **Owner:** a design
  session resolving the mutual deferral (engine boot-sequencing home), before the
  first our-engine-hosted MP boot slice.

- **LOAD-Q13 — Where the release-build missing-export `ERR_FATAL` physically fires
  (mechanism; contract settled).** The user-settled *contract* (§ Load / unload /
  restart amendment 2026-07-03, item 18): a **found library missing `vmMain`/`dllEntry`**
  raises `com_error(ERR_FATAL, …)` in `cfg(not(debug_assertions))` (release) and keeps
  `com_printf` + `None` in debug — faithful to `unix_main.c:431-436`'s `#ifdef NDEBUG`
  split (porting-rules §20). What is **unsettled** is the *mechanism*: `sys_load_dll`
  lives in `crates/native/platform` (tier −1), which **cannot name
  `mp_engine_qcommon::com_error`** (uphill edge — the checkpoint-2 `EngineSlot`-injection
  class). Candidates, none picked: (a) an **injected fatal hook** handed to the loader at
  load time (à la the settled `systemCalls`/`ctx` injection); (b) a **platform-local
  sys-fatal** primitive in `native/platform` (print + `exit`, no `com_error`); or (c) a
  **three-state loader return** (found / not-found / found-but-bad-export) with the
  cfg-gated `com_error` raised by the caller in `mp_engine_qcommon::load_module`. Does
  **not** change the frozen `sys_load_dll` signature — only where the release-fatal is
  emitted. **Owner:** slice-0 wiring (checkpoint-5 finding 23); not a Slice-0 blocker
  (SEAM-D7's Slice 0 is hosted inside a real engine, which owns the loader).

**Resolved (2026-07-03 session, skeleton-seed findings — supersession recorded in
LOAD-D8's 2026-07-03 amendment):** LOAD-Q11 → **EngineSlot load-time injection**.
The per-slot cell no longer holds a typed `Cell<*mut Engine>` that would force the
forbidden `mp_engine_qcommon → mp_engine_core::Engine` uphill edge; it holds the
**injected** `EngineSlot { ctx: *mut c_void, syscall: SlotSyscall }`
(`engine-seam.md:545-570,547`) — an opaque `ctx` + syscall fn pointer handed in at
module-load, mirroring Raven's `VM_Create` storing its received `systemCalls`
argument (`vm.cpp:471-472,506`). `mp_engine_qcommon` therefore never names the
engine aggregate (`engine-seam.md:555-557`), the cell's construction is settled
(built at `load_module` from its own `ctx`/`system_calls` params), and
`engine_slot.rs`/`module_slot.rs` compile with no uphill edge — the exact
compilability blocker LOAD-Q11 named. The earlier "no cited doc settles how the
below-facade crate names `*mut Engine`" framing is withdrawn: it no longer names one.

**Resolved (2026-07-03 session), now a Decision amendment:** LOAD-Q10 →
**LOAD-D11** (2026-07-03 amendment). `load_module` returns **`Option<SlotId>`**: a
fresh-slot not-found (`sys_load_dll(...) -> None`) yields `None`, mirroring
`sys_load_dll`'s own `Option` contract, and the **caller** owns the non-uniform
fatal disposition — faithful to Raven, where `VM_Create` returns `NULL`
non-fatally and each caller `Com_Error`s differently (`SV_InitGameProgs`/`CL_InitUI`
`ERR_FATAL`, `CL_InitCGame` `ERR_DROP`; `sv_game.cpp:1750-1752`,
`cl_ui.cpp:1479-1481`, `cl_cgame.cpp:1772-1774`). The earlier "not derivable from
oracle ground truth" framing is withdrawn (oracle derives it at the caller). The two
in-`VM_Create` `ERR_FATAL`s (bad-parms, slot-full) stay internal to `load_module`
per the original LOAD-D11 mechanism.

**Resolved (2026-07-02 escalation session), now Decisions:** LOAD-Q2 → **LOAD-D7**
(`Sys_UnpackDLL` in scope, deferred to a post-B2 MP-server slice, Slice 0 stubs
it); LOAD-Q3 → **LOAD-D8** (`ModuleRegistry` slot API frozen against `VM_Create`);
LOAD-Q4 → **LOAD-D6** (raw ABI aliases relocate to `crates/native/platform`,
`abi-transport` re-exports them).

**Resolved (2026-07-02 post-nap session), now Decisions:** LOAD-Q6 → **LOAD-D9**
(`SearchStep::FsPath` carries resolved paths; policy built at the
`mp_engine_qcommon` call site, `native/platform` stays cvar-free); LOAD-Q5 →
**LOAD-D10** (host-side wasm types live in `crates/mp/engine/wasm-host`); LOAD-Q7
→ **LOAD-D11** (`load_module`'s `ERR_FATAL` is a direct receiverless `com_error`
panic; the return was **later widened to `Option<SlotId>`** by the 2026-07-03
amendment resolving LOAD-Q10); LOAD-Q8 → **LOAD-D12b** (`RestartKind` threaded
through `ModuleRegistry::restart` as a caller-supplied `kind` parameter).

**Referenced but owned elsewhere (not re-opened here):** the `GetModuleAPI`
OpenJK-native handshake contract (**SEAM-Q7**) remains open in `engine-seam.md`.
The raw inbound syscall trampoline a Rust engine hands a hosted DLL (**SEAM-Q9**)
is now **resolved** upstream as **SEAM-D11** (one `extern "C-unwind"` C-shim
trampoline per module slot reading a per-slot **injected** `EngineSlot { ctx,
syscall }` — amended 2026-07-03, `engine-seam.md:545-570`; the injection is what
lets `mp_engine_qcommon` avoid the uphill `Engine` name, resolving LOAD-Q11); the
module-shell crate
identity + dependency edges (**SEAM-Q8**) is **resolved** upstream as **SEAM-D10**
(`crates/jampgame` thin cdylib shell), which this doc's Slice 0 hooks anchor on.
This doc's matrix and seam depend on those resolutions but does not settle them.

The **load call site** — where `ModuleRegistry::load_module` is actually invoked
(the `SV_InitGameProgs`-equiv function, `oracle/codemp/server/sv_game.cpp:1750`)
— splits into a *settled* and an *unsettled* half. Its **trigger** is settled by
lifecycle.md (FROZEN, 2026-07-03): the game module loads at **map spawn**
(`SV_SpawnServer → SV_InitGameProgs`, **post-Slice-0**), not at engine boot — Slice-0
`Com_Init` only default-constructs the empty registry at step-30 `VM_Init`
(`lifecycle.md:82,187-215,222`). Its **crate/signature** (and where the `None`-match
lives) is settled by **neither** doc: lifecycle.md punts `SV_InitGameProgs` module-load
mechanics **back** to this doc (`lifecycle.md:69-70`) while this doc's Scope punts the
call site to lifecycle.md + SEAM-D7 — a mutual deferral tracked as **LOAD-Q12**. The
loader/registry surface this doc freezes is standalone-**buildable** (Verification 1);
driving an end-to-end our-engine boot waits on LOAD-Q12. (SEAM-D7's Slice-0 model
hosts our module **inside a real/OpenJK engine**, so the *our-engine*
`SV_InitGameProgs`-equiv is not itself a Slice-0 deliverable — LOAD-Q12 is
non-blocking for Slice 0.)
