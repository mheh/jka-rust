# Module Loading Design
Status: DRAFT     Supersedes: none
Decision prefix: LOAD     Ledger deps: DEC-05, DEC-07, DEC-09

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
  after native parity), DEC-07 (SP cgame/ui via the vmachine shim), DEC-09
  (verification layers).
- `docs/architecture/engine-seam.md` — the **typed call/dispatch** side (SEAM-D1
  compile-time-per-artifact vs runtime-per-module transport, SEAM-D4 pointer-word
  interpretation, `ModuleTransport` enum, `Execute`/`Dispatch`, `CEngine`,
  `SharedGameData`). This doc supplies the loader that produces the handles that
  doc dispatches through; the two are duals.
- `docs/architecture/two-island-model.md` — STATE-D3 (`extern "C-unwind"`).

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
  `docs/architecture/lifecycle.md` (pending). This doc records the Raven
  create/destroy *cadence* only as ground truth for the restart semantics it
  freezes.
- **Which physical crate hosts each module shell** (`ENGINE` + live exports)
  and its dependency edges → engine-seam.md **SEAM-D10** (resolved upstream:
  `crates/jampgame` is the thin cdylib shell, deps `abi_transport` + `mp_game`).
  This doc anchors its Slice 0 call sites on that shell (Slice hooks); it does
  not restate the shell contract.
- **The `GetModuleAPI` OpenJK-native handshake contract** → SEAM-Q7 (engine-seam,
  open). This doc places it in the parity matrix; it does not define its body.
- **The raw inbound syscall trampoline** an engine hands a hosted DLL → SEAM-Q9
  (engine-seam, open).

## Raven ground truth

### MP native-DLL load chain

`vm_t *VM_Create( const char *module, systemCalls, vmInterpret_t interpret )`
(`oracle/oracle/codemp/qcommon/vm.cpp:471-472`) is the entry. It reuses a live
slot by name (`vm.cpp:485-489`), else allocates the first free slot of
`vmTable[MAX_VM]` (`#define MAX_VM 3`, `vm.cpp:28-29`; fatal if full,
`vm.cpp:499-500`), stores `systemCall` (`vm.cpp:505-506`), then attempts a
native load: `Sys_LoadDll(module, &vm->entryPoint, VM_DllSyscall)`
(`vm.cpp:515-518`). Success returns immediately; failure falls through to the
QVM path — **out of scope (DEC-05.4)** (`vm.cpp:519-524`, non-fatal).

`Sys_LoadDll` (Win32, `oracle/oracle/codemp/win32/win_main.cpp:811-887`):

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
   Because it extracts from a pk3 and touches filesystem ownership, porting it is
   an unresolved fork — **LOAD-Q2**.
3. **Search order**, first hit wins (`win_main.cpp:855-873`): bare
   `LoadLibrary(filename)` (CWD / default DLL search, `:855`); then
   `FS_BuildOSPath(fs_basepath, fs_game, filename)` (`:858-863`); then
   `FS_BuildOSPath(fs_cdpath, fs_game, filename)` **only if `fs_cdpath[0]`**
   (`:866-869`); else `return NULL` (`:871-873`). **No `fs_homepath`
   fallback.** `FS_BuildOSPath` builds `"<base>/<game>/<qpath>"`
   (`oracle/oracle/codemp/qcommon/files.cpp:479-498`). **Win32-only direct
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
   mirrors this with `dlopen`/`dlsym` (`oracle/oracle/codemp/unix/unix_main.c:
   384,421,428,444`), filename `"%si386.so"` (`unix_main.c:346`; `ppc`/`axp`/
   `mips` variants, `-debug` infix in debug builds).

**macOS reality:** no `codemp/mac/` directory exists — **MP has no Mac
dynamic-loading backend in oracle** (dossier §3). SP's
`oracle/oracle/code/mac/mac_main.c:65-73` `Sys_LoadDll` is a hard-linked no-op
stub (`*entryPoint = vmMain; return (void*)1;`).

### MP create/destroy cadence (ground truth for restart, not lifecycle)

- **jampgame** — `SV_InitGameProgs` creates `VM_Create("jampgame", ...)`
  (`oracle/oracle/codemp/server/sv_game.cpp:1750`); `SV_ShutdownGameProgs`
  `VM_Call(GAME_SHUTDOWN); VM_Free(gvm)` (`sv_game.cpp:1666-1673`). Every normal
  map change **destroys+recreates**: `SV_ShutdownGameProgs()` (`sv_init.cpp:484`)
  then `SV_InitGameProgs()` (`sv_init.cpp:662`). Only `map_restart` uses the
  in-place `VM_Restart` via `SV_RestartGameProgs` (`sv_game.cpp:1712-1715`,
  `sv_ccmds.cpp:296`). Torn down for good at `SV_Shutdown` (`sv_init.cpp:946`).
- **cgame** — `CL_InitCGame` `VM_Create("cgame", ...)`
  (`oracle/oracle/codemp/client/cl_cgame.cpp:1771`); `CL_ShutdownCGame`
  `VM_Free(cgvm)` (`cl_cgame.cpp:601-603`). Destroyed+recreated on every map
  load via `CL_DownloadsComplete → CL_FlushMemory → CL_ShutdownAll /
  CL_InitCGame` (`cl_main.cpp:1497,1501`), and on `vid_restart`
  (`cl_main.cpp:1322,1324,1362`). Not freed by plain disconnect.
- **ui** — `CL_InitUI` `VM_Create("ui", ...)`
  (`oracle/oracle/codemp/client/cl_ui.cpp:1478`), version-checked against
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
(`oracle/oracle/code/win32/win_main.cpp:478-547`) loads `"jagamex86.dll"`
(`:489`) — search is **narrower than MP's**: `<cwd>/<debugdir>/jagamex86.dll`
(`:515`) then `<cwd>/jagamex86.dll` (`:524`), no `fs_*` cvars; both fail →
`Com_Error(ERR_FATAL, "Couldn't load game")` (`:536`). Symbol:
`GetProcAddress(game_library, "GetGameAPI")` (`:540`), then `return
GetGameAPI(parms)` (`:546`). The game import/export is a **struct of typed
fn-pointers**, not numbered syscalls (`code/game/g_public.h`), version-checked
engine-side (`sv_game.cpp:682-684`).

Immediately after, `SV_InitGameProgs` fakes `VM_Create("cl")`
(`code/server/sv_game.cpp:676-679`) — **not a second DLL**. That is SP's
fake-VM shim `oracle/oracle/code/client/vmachine.h:72-84`: for module `"cl"` it
calls `Sys_LoadCgame(&cgvm.entryPoint, VM_DllSyscall)`, which
`GetProcAddress`es `dllEntry`/`vmMain` on the **same** `game_library` handle
already holding `jagamex86.dll` (`win_main.cpp:557-570`) — cgame logic is
compiled *into* `jagamex86.dll` (`code/game/game.vcproj`). Console/Mac builds
resolve the same `GetGameAPI`/`vmMain`/`cg_dllEntry` as plain `extern` symbols
with zero `LoadLibrary` (`win_main_console.cpp:542-567`, `mac_main.c:35-73`).
**UI is always statically linked into the exe in every SP build** — no UI DLL
project exists anywhere in SP. Conclusion (dossier §4): `jagamex86.dll` is the
only ever-loadable SP module; cgame is never its own DLL; UI is never a module.

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
`OnceLock`, `cgvm.entryPoint`, and the eliminated `currentVM`/`lastVM` are owned
by `engine-seam.md`'s table and only cross-referenced here to avoid duplication,
porting-rules §4 / doc-standards §4).

| Raven global | oracle cite | Rust owner (crate::Type.field) | constructed by | threaded via |
|---|---|---|---|---|
| `vmTable[MAX_VM]` (`MAX_VM = 3`) | `qcommon/vm.cpp:28-29` | `mp_engine_qcommon::ModuleRegistry.slots: [Option<LoadedModule>; MAX_VM]` — **per-slot only, no current-module global** (LOAD-D5); home crate `mp_engine_qcommon` mirrors `vm.cpp`'s subsystem (LOAD-D8, engine-seam SEAM-D10) | `ModuleRegistry::load_module` (FROZEN — LOAD-D8: slot reuse-by-name `vm.cpp:485-489`, first free slot `vm.cpp:494`, `Com_Error(ERR_FATAL)` when all `MAX_VM` full `vm.cpp:499-500`) | dispatcher `engine` arg (engine-seam) |
| `vm->dllHandle` | `qcommon/vm_local.h:111-146` | `LoadedModule.lib: libloading::Library` in the slot | `sys_load_dll` | held in slot; dropped on unload |
| `vm->entryPoint` | `qcommon/vm_local.h:123` | `LoadedModule.entry: RawVmMain` in the slot | handshake in `sys_load_dll` | passed to `Dispatch` (engine-seam) |
| `currentVM`, `lastVM` | `qcommon/vm.cpp:24-25` | **eliminated** (LOAD-D5; owned/justified in engine-seam state table) — the `VM_Free` clobber (`vm.cpp:624-625`) is structurally unreproducible | — | — |
| `gvm` (jampgame slot) | `server/sv_game.cpp:1750` | `ModuleRegistry` slot; server state holds the `SlotId` | `SV_InitGameProgs`-equiv | server state |
| `cgvm`, `uivm` (MP slots) | `client/cl_cgame.cpp:1771`, `client/cl_ui.cpp:1478` | `ModuleRegistry` slots; client state holds their `SlotId`s | `CL_InitCGame` / `CL_InitUI`-equiv | client state |
| SP `game_library` `HINSTANCE` | `code/win32/win_main.cpp:459` | only present in the **i686-windows drop-in hosting** scenario (matrix row 3) as a `LoadedModule`; our own SP is fully static (DEC-07) → no handle | `Sys_GetGameAPI`-equiv | — |
| SP `cgvm`/`uivm` (vmachine shim) | `code/client/vmachine.h:54-55` | static dispatch handles, **no load** (DEC-07); owned as `cgvm.entryPoint` in engine-seam state table | static link | dispatcher arg (engine-seam) |

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
    /// win_main.cpp:826 ("x86.dll"); unix_main.c:346 ("i386.so"); macOS: ".dylib".
    pub suffix: &'static str,
}

/// A mode's search policy — a value, constructed by each mode's app (LOAD-D1).
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
    /// `FS_BuildOSPath(<base_cvar>, <gamedir_cvar>, filename)` (files.cpp:479-498).
    /// MP order = basepath/fs_game then cdpath/fs_game; NO homepath (win_main.cpp:858-869).
    FsPath { base_cvar: &'static str, gamedir_cvar: &'static str },
    /// SP: `<cwd>/<subdir?>/filename` (win_main.cpp:515,524). No fs_* cvars.
    CwdRelative { subdir: Option<&'static str> },
}
```

Each mode's app owns a **named** policy constructor `module_search_policy()`
(LOAD-D6): `crates/mp/app::module_search_policy()` (in `openjk` / `openjkded`)
and `crates/sp/app::module_search_policy()` (in `openjk_sp`), per
`docs/workspace-architecture.md` (`mp/app` links the engine subsystems + `mp/abi`
and hosts the module cdylibs). This doc freezes the per-mode policy **value** each
returns — its `naming` / `direct_first` / `steps` — which is what Slice 0 needs;
*when* that fn is called during boot is `lifecycle.md`'s (pending). The MP
`direct_first` split is per-platform (LOAD-D1, amended 2026-07-02):

```rust
// crates/mp/app::module_search_policy()
// Win32 (win_main.cpp:855-869): LoadLibrary-direct, then basepath/fs_game, then cdpath/fs_game.
// Unix  (unix_main.c:361-396):  basepath/fs_game then cdpath/fs_game; the cwd/installdir
//   dlopen is #if 0'd ("bk010205 - do not load from installdir", unix_main.c:373) → NO direct probe.
ModuleSearchPolicy {
    naming: ModuleNaming { suffix: /* platform: "x86.dll" | "i386.so" | ".dylib" */ },
    // Win32: true (win_main.cpp:855); Unix: false (unix_main.c:361-373 is #if 0'd).
    direct_first: cfg!(windows),
    steps: vec![
        SearchStep::FsPath { base_cvar: "fs_basepath", gamedir_cvar: "fs_game" },
        SearchStep::FsPath { base_cvar: "fs_cdpath",   gamedir_cvar: "fs_game" }, // skipped if empty
    ],
}
// crates/sp/app::module_search_policy()
// SP (win_main.cpp:515,524): cwd/<debugdir> then cwd, no fs_* cvars, no direct probe.
ModuleSearchPolicy {
    naming: ModuleNaming { suffix: /* "x86.dll" */ },
    direct_first: false,
    steps: vec![
        SearchStep::CwdRelative { subdir: Some(/* release|shdebug|debug */) },
        SearchStep::CwdRelative { subdir: None },
    ],
}
```

### Load / unload / restart

```rust
/// A live native module: the library handle plus its resolved entrypoints,
/// held in a ModuleRegistry slot (State ownership). Faithful to vm->dllHandle +
/// vm->entryPoint (vm_local.h:123).
pub struct LoadedModule {
    lib: libloading::Library,   // vm->dllHandle (win_main.cpp:855-863)
    entry: RawVmMain,           // "vmMain" (win_main.cpp:880); RawVmMain defined here in
                                // native/platform (LOAD-D6), re-exported by abi-transport
}

/// Faithful to Sys_LoadDll (win_main.cpp:811-887) MINUS the pure-server
/// `Sys_UnpackDLL` pre-step (`:849-852`), which is IN SCOPE but DEFERRED to a
/// later MP-server slice (LOAD-D7 — it needs FS_ReadFile/FS_FileIsInPAK/
/// FS_FOpenFileWrite, which land with the filesystem port B2). Slice 0 is
/// non-pure and stubs it: where the pre-step would run the porter leaves a
/// `//TODO: Port Sys_UnpackDLL` + `// Source:` marker (porting-rules §14 —
/// unported deps are explicit, never a silent no-op), NOT a silently-swallowed
/// step. Namely: apply naming, walk the policy, `GetProcAddress`/`dlsym`
/// "dllEntry"+"vmMain" (both required), call `dllEntry(syscall)`. `None` = not
/// found (Raven's QVM fallback is out of scope, DEC-05.4; the caller decides
/// fatal-vs-skip per mode).
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

**Alias home (LOAD-D6).** The raw ABI aliases named above — `RawVmMain`
(`LoadedModule.entry`) and `RawSyscall` (`sys_load_dll`'s `syscall`) — **live in
`crates/native/platform`** (base tier: they are Raven-free platform-ABI
vocabulary). `abi-transport` takes a downhill dep on `native/platform` and
**re-exports** them from `entrypoints.rs`, so existing `abi-transport` consumers
are unaffected and no tier inversion occurs. The frozen loader signatures
therefore compile in `native/platform` with no new outward edge.
`workspace-architecture.md` gains the `abi-transport → native/platform` edge
(updated there separately).

### Module registry — `crates/mp/engine/qcommon`

`sys_load_dll` / `unload_module` above are the per-artifact primitives. The slot
registry that owns them — the `vmTable[MAX_VM]` replacement — is FROZEN (LOAD-D8)
against `VM_Create`'s slot semantics; its home crate `mp_engine_qcommon` mirrors
`vm.cpp`'s subsystem (engine-seam SEAM-D10).

```rust
// crates/mp/engine/qcommon — the slot registry (LOAD-D8), replaces vmTable[MAX_VM].
pub struct SlotId(u32);   // index into slots[0..MAX_VM]

pub struct ModuleRegistry {
    slots: [Option<LoadedModule>; MAX_VM],   // MAX_VM = 3 (vm.cpp:28-29)
}

impl ModuleRegistry {
    /// VM_Create slot semantics (vm.cpp:471 region): reuse a live slot whose
    /// module name matches (vm.cpp:485-489, returned as-is, NO reload), else the
    /// first free slot (vm.cpp:494), else Com_Error(ERR_FATAL) when all MAX_VM
    /// slots are full (vm.cpp:499-500). A fresh slot runs sys_load_dll.
    pub fn load_module(&mut self, policy: &ModuleSearchPolicy, name: &str,
                       syscall: RawSyscall) -> SlotId;

    /// VM_Free (vm.cpp:605-610): unload_module the slot's module, clearing it.
    /// No global currentVM/lastVM clobber (LOAD-D5).
    pub fn unload(&mut self, slot: SlotId);

    /// Native VM_Restart = drop+recreate in place (vm.cpp:398-409): unload then
    /// reload the same slot (RestartKind::DropRecreate). Wasm may substitute the
    /// in-place reset (LOAD-D2) behind the same call.
    pub fn restart(&mut self, slot: SlotId, policy: &ModuleSearchPolicy,
                   name: &str, syscall: RawSyscall);
}
```

### Host-side wasm pointer shape (LOAD-D3)

A guest linear-memory offset is a `u32`, **never** a host address. Accessors are
bound to the module `Memory` and **re-resolve per access** (never cache a base —
`memory.grow` invalidates it) with **explicit bounds checks** (wasm memories
aren't power-of-2, so `VM_ArgPtr`'s `& dataMask` mask, `vm.cpp:652`, is replaced
by a range check; the `dataBase + offset` translation, `vm.cpp:649`, is the
precedent).

```rust
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

**Not frozen here — the crate home of `WasmPtr<T>` / `ModuleMemory`.** LOAD-D3
fixes their *shape* but not their *location*: they are host-side (bound to a
wasmtime `Memory`, so native — not a `wasm32` guest type, so they cannot live in
the `#[cfg(target_arch = "wasm32")]` arm of `abi-transport::entrypoints`,
LOAD-D4), yet `docs/workspace-architecture.md` declares **no** wasmtime-host tier
or crate, and — unlike every other Seam subsection — this code block carries no
crate-path comment. Which crate hosts them is an open fork, **LOAD-Q5**, on the
same footing as LOAD-Q4. First exercised by the wasm host slice (post native
parity, DEC-05.5); it does not block any native slice.

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
match is SEAM-D9's per-module-shell pattern, its identity SEAM-Q8 (engine-seam,
open) — this doc only fixes that whichever crate it is, it selects the
`native`/`wasm` arm above by its `--target`.

### SP linkage surface (DEC-07, LOAD-D5)

Our SP **always** uses the console/Mac fully-static shape — `Sys_LoadCgame` /
`GetProcAddress` are never exercised (dossier §4 conclusion):

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

### DEC-05 drop-in parity matrix (both directions)

| # | Host | Module | Platform / build target | Transport | Notes |
|---|---|---|---|---|---|
| 1 | Rust engine | Rust module | any native (x86_64 / i686 / arm64) cdylib | NativeDll | core scenario (DEC-05.1); `native/platform` loader, LOAD-D1 |
| 1s | Rust engine | Rust module | same crate, statically linked | Static | our-engine hosting; no load step (engine-seam SEAM-D1) |
| 1w | Rust wasmtime host | Rust module | wasm32 | Wasm | DEC-05.5; lands **after** native parity |
| 2a | retail `jamp` 1.01 | Rust module | **i686-pc-windows** cdylib (`…x86.dll`) | NativeDll | drop-in (DEC-05.2); seam structs oracle-exact |
| 2b | OpenJK native | Rust module | native cdylib per platform | NativeDll (+ `GetModuleAPI`) | DEC-05.2; game-private layout per-host (NOTES.md:65-68); `GetModuleAPI` contract SEAM-Q7 |
| 3 | Rust engine | real / mod DLL (JA+, MBII, …) | **i686-pc-windows engine only** | NativeDll (hosting) | DEC-05.3; 32-bit PE; raw inbound trampoline SEAM-Q9 |
| gate | `cargo build` | Rust module crates | wasm32-unknown-unknown | — | standing CI compile-gate from day one (DEC-05.5) |

## Verification strategy

Per DEC-09, native track (porting-rules §E — green at every commit, one
file/function per commit):

1. **Loader TU tests** (DEC-09.1 pattern): search-order goldens — construct
   fixture directories that plant a fake artifact at each candidate location and
   assert `sys_load_dll` walks the MP **Win32** order (`LoadLibrary`-direct →
   basepath → cdpath, no homepath), the MP **Unix** order (basepath → cdpath, **no
   direct probe**), and the SP order (`cwd/<debugdir>` → `cwd`) exactly,
   first-hit-wins, matching `win_main.cpp:855-869` / `unix_main.c:361-396` /
   `:515,524`. Naming-table
   goldens assert `"jampgame" → jampgamex86.dll` / `jampgamei386.so` /
   `jampgame.dylib` per platform.
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
  `RestartKind::DropRecreate` (LOAD-D2). No wasm, no cgame/ui. **Slice 0's
  compilable skeleton is gated on three still-open forks** (all escalated, none
  self-resolvable here): SEAM-Q8 — which shell crate's `lib.rs` hosts the
  `sys_load_dll` call site (same as engine-seam Slice 0); LOAD-Q4 — until the
  tier-model fork resolves, `sys_load_dll`'s `RawSyscall` and
  `LoadedModule.entry: RawVmMain` aliases cannot be named or compiled inside
  `crates/native/platform`; and LOAD-Q3 — the `SV_InitGameProgs`-equiv boot needs
  a frozen `ModuleRegistry::load_module` / `SlotId` call to register the loaded
  `jampgame` slot (the per-artifact `sys_load_dll` primitive is frozen; the
  registry method that yields a `SlotId` is not).
- **Client slices** add cgame/ui loading (same MP policy, same drop+recreate
  cadence, dossier §1) and their registry slots.
- **SP slice** wires the `GetGameAPI` table + vmachine shim dispatch — no loader
  path exercised (DEC-07, LOAD-D5).
- **Wasm host slice** (post native-parity, DEC-05.5): the `wasm` entrypoint arm,
  `ModuleMemory`/`WasmPtr`, and the restart-equivalence parity test.

## Open questions

**Open — returns to a design session:**

- **LOAD-Q1 — Exact macOS artifact filename.** LOAD-D1 settles the macOS naming
  entry as our `.dylib` extension, but oracle has **no** Mac MP loader (dossier
  §3), so the precise base string — whether the module name carries an arch infix
  (e.g. OpenJK's `…arm64.dylib` / `…x86_64.dylib`) or a bare `…​.dylib` — cannot
  be derived from oracle ground truth and must match whatever OpenJK-style host
  we intend to interoperate with (a DEC-05.2 interop detail). Until resolved a
  porter leaves the macOS `ModuleNaming.suffix` unset (`//TODO: Port` per
  porting-rules §14) rather than hardcoding a value; the Win32/Unix entries and
  the whole native-parity path are unaffected. Escalated; not needed for Slice 0
  (i686-windows / native-Linux only).

- **LOAD-Q2 — Pure-server `Sys_UnpackDLL` pre-step.** `Sys_LoadDll` runs
  `if (!Sys_UnpackDLL(filename)) return NULL;` (`win_main.cpp:849-852`) before any
  `LoadLibrary`; `Sys_UnpackDLL` (`win_main.cpp:762-800`) extracts the DLL from a
  pk3 to disk when the server is pure (`FS_ReadFile` / `FS_FileIsInPAK` /
  `FS_FOpenFileWrite` / `FS_Write`) and aborts the load on failure. It is
  MP-Win32-only (no Unix or SP equivalent in oracle) and touches filesystem
  ownership. No LOAD decision covers whether to **port** it, **drop** it, or
  **delegate** it as a non-goal to a filesystem-owning doc; `sys_load_dll`'s seam
  deliberately omits it (Seam definition) rather than silently swallowing it.
  Escalated — a porter implementing `sys_load_dll` faithfully otherwise hits an
  undocumented fork. For an on-disk (non-pak) DLL the function returns `true`
  without writing (`FS_FileIsInPAK == -1`, `:774-779`), so a non-pure local load
  is unaffected and Slice 0's on-disk `jampgame` load does not exercise it; the
  fork bites only pure-server hosting. **Porter action until the fork resolves:**
  emit a `//TODO: Port Sys_UnpackDLL` marker + `// Source:
  oracle/oracle/codemp/win32/win_main.cpp:762-800` where the pre-step would run —
  porting-rules §14 forbids a silently-swallowed step; this is a doc-mandated
  marker, not a decision to drop the behavior.

- **LOAD-Q3 — `ModuleRegistry::load_module` signature + slot contract.** The
  State-ownership `ModuleRegistry` is "constructed by the loader on each
  `load_module`", but the Seam freezes only the per-artifact `sys_load_dll` /
  `unload_module`. The slot-allocating `load_module` layer's exact signature and
  its two cited Raven behaviors — slot reuse-by-name returns the live slot with no
  reload (`vm.cpp:485-489`) and all-`MAX_VM`-slots-full is fatal
  (`vm.cpp:499-500`) — are not frozen. LOAD-D5 permits host-side deviation here
  (the registry is bookkeeping, not an ABI-crossing struct), so whether to
  preserve name-reuse / fatal-overflow verbatim or model them differently is a
  design choice no LOAD decision settles. Escalated — freezing
  `ModuleRegistry::load_module`'s signature and its reuse/overflow contract
  returns to a design session.

- **LOAD-Q4 — Crate home of the raw ABI aliases + the loader's edge to
  `abi-transport`.** The FROZEN loader seam names abi-transport's raw ABI aliases
  — `RawVmMain` (`LoadedModule.entry`) and `RawSyscall` (`sys_load_dll`'s
  `syscall` param), defined at `crates/abi-transport/src/entrypoints.rs:5,10`. But
  LOAD-D4 pins the loader (and all libloading/OS types) inside
  `crates/native/platform`, which `docs/workspace-architecture.md` places at
  **tier -1** ("genuinely Raven-free, cross-mode … used by everything",
  `workspace-architecture.md:88`), **below** the transport tier (`abi-transport`,
  used by "both `*/abi`, both engines", `:89`). Reaching `RawSyscall`/`RawVmMain`
  from the loader therefore requires a `native/platform → abi_transport` edge that
  neither this doc nor workspace-architecture establishes, and that inverts the
  tier model — a cycle the moment `abi-transport` takes the native dep the "used
  by everything" framing implies (both crates' `Cargo.toml`s currently declare
  zero dependencies). No LOAD decision covers **where the raw ABI aliases live** or
  **which crate hosts the loader relative to `abi-transport`**, so a porter
  implementing `sys_load_dll` hits an undocumented fork: import `abi-transport`
  into `native/platform` (invert the tiers) vs. redefine the raw aliases locally
  (duplicate an ABI type). This is the direct analog of SEAM-Q8 (shell-crate
  identity + dependency edges, engine-seam) and is escalated on the same footing;
  its resolution — a new sanctioned edge, or a relocation of the aliases and/or
  loader — is a design choice no LOAD decision settles. Not needed to freeze the
  loader's behavior (naming, search order, handshake, restart), but required
  before `crates/native/platform` can name these types.

- **LOAD-Q5 — Crate home of the host-side wasm types (`WasmPtr<T>` /
  `ModuleMemory`).** LOAD-D3 freezes the *shape* of the host-side wasm pointer
  types, but — unlike every other Seam subsection, which names its owning crate
  (`crates/native/platform`, `crates/abi-transport/src/entrypoints.rs`) — the
  "Host-side wasm pointer shape" code block names none, and
  `docs/workspace-architecture.md`'s tier table declares **no** tier or crate for
  a wasmtime host (the `Memory` handle these accessors bind to). They are
  host-side (the engine running wasmtime is native), so they cannot live in the
  `#[cfg(target_arch = "wasm32")]` guest arm of `abi-transport::entrypoints`
  (LOAD-D4). Whether they belong in `crates/native/platform`, a new wasmtime-host
  crate, or elsewhere — and the corresponding addition to the crate graph — is a
  fork no LOAD decision or standing doc settles. Escalated on the same footing as
  LOAD-Q4. Not needed for Slice 0 or any native-parity slice; first exercised by
  the wasm host slice (post native parity, DEC-05.5).

**Referenced but owned elsewhere (not re-opened here):** the module-shell crate
identity + dependency edges (**SEAM-Q8**), the `GetModuleAPI` OpenJK-native
handshake contract (**SEAM-Q7**), and the raw inbound syscall trampoline a Rust
engine hands a hosted DLL (**SEAM-Q9**) all live in `engine-seam.md`. This doc's
matrix and seam depend on their resolutions but does not settle them.
