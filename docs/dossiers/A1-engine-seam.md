# A1 — Engine Seam Dossier (ground truth for design session)

Scratch/working doc for the engine-seam design session. Read-only survey of
`oracle/oracle/` + current `crates/`. Every behavioral claim cites
`oracle/oracle/<path>:<line>`; Rust-state claims cite `crates/<path>:<line>`.

---

## 1. MP syscall mechanics

### 1.1 vmMain shapes (all 3 modules)

Shared convention: `int vmMain(int command, int arg0..arg11)` — command word +
12 argument words, all `int`-sized regardless of host word width.

- **`game/g_main.c:515`** — `int vmMain( int command, int arg0, ..., int arg11 )`.
  Switch on `GAME_*` (`:516`+): `GAME_INIT` (`:517`, `G_InitGame(arg0,arg1,arg2)`),
  `GAME_SHUTDOWN` (`:520`), `GAME_CLIENT_CONNECT` (`:523`), `GAME_CLIENT_THINK`
  (`:525`), `GAME_CLIENT_USERINFO_CHANGED` (`:528`), `GAME_CLIENT_DISCONNECT`
  (`:531`), `GAME_CLIENT_BEGIN` (`:534`), `GAME_CLIENT_COMMAND` (`:537`),
  `GAME_RUN_FRAME` (`:540`), `GAME_CONSOLE_COMMAND` (`:543`), `BOTAI_START_FRAME`
  (`:545`), `GAME_ROFF_NOTETRACK_CALLBACK` (`:547`, dereferences `&g_entities[arg0]`
  and casts `arg1` to `const char*` — valid only because native-DLL args are real
  host pointers), `GAME_SPAWN_RMG_ENTITY` (`:550`), plus ICARUS callback cases
  (`GAME_ICARUS_PLAYSOUND` `:557`+) that read a **shared-memory struct**
  (`gSharedBuffer`) instead of `arg0..arg11` — an alternate high-arity idiom.
- **`cgame/cg_main.c:190`** — same signature. `CG_INIT` (`:194`), `CG_SHUTDOWN`
  (`:197`), `CG_CONSOLE_COMMAND` (`:199`), `CG_DRAW_ACTIVE_FRAME` (`:201`),
  `CG_CROSSHAIR_PLAYER`/`CG_LAST_ATTACKER` (`:204`,`:206`), `CG_KEY_EVENT`/
  `CG_MOUSE_EVENT` (`:208`,`:211`), `CG_POINT_CONTENTS` (`:220`), `CG_GET_GHOUL2`
  (`:227`, returns `(int)cg_entities[arg0].ghoul2` — truncates pointer to `int`,
  in-source comment flags it as VM-unsafe garbage), `CG_TRACE`/`CG_G2TRACE`
  (`:239`,`:243`, delegate to `C_Trace()`/`C_G2Trace()` pulling real args from a
  shared struct, not `arg0..arg11`), `CG_INCOMING_CONSOLE_COMMAND` (`:250`,
  default return `1`).
- **`ui/ui_main.c:579`** — same signature. `UI_GETAPIVERSION` (`:581`, returns
  `UI_API_VERSION` — version-negotiation case unique to UI), `UI_INIT` (`:584`),
  `UI_SHUTDOWN` (`:588`), `UI_KEY_EVENT`/`UI_MOUSE_EVENT` (`:592`,`:596`),
  `UI_REFRESH` (`:600`), `UI_IS_FULLSCREEN` (`:604`), `UI_SET_ACTIVE_MENU`
  (`:607`), `UI_CONSOLE_COMMAND` (`:611`), `UI_DRAW_CONNECT_SCREEN` (`:614`),
  `UI_HASUNIQUECDKEY` (`:617`, hardcoded `qtrue`), `UI_MENU_RESET` (`:619`);
  falls through to `return -1` (`:623`) for unhandled commands — smallest of the
  three switches, no shared-buffer cases.

Cross-module: game + cgame both escape the 12-word cap via a shared-struct
idiom for high-arity calls; UI never needs to.

### 1.2 The syscall fn-ptr type

`game/g_syscalls.c:8` — `static int (QDECL *syscall)( int arg, ... ) =
(int (QDECL *)( int, ...))-1;` (poisoned until wired). `game/g_syscalls.c:14` —
`void dllEntry( int (QDECL *syscallptr)( int arg,... ) ) { syscall = syscallptr; }`,
called once by the engine at load time (§1.5/1.6). Every `trap_*` wrapper calls
the module-local variadic `syscall(...)`; callee (`VM_DllSyscall`, §1.6)
reinterprets the varargs region as a flat `int[]` (relies on cdecl stack
contiguity).

### 1.3 Four representative trap_* wrappers

- **Scalar** — `game/g_syscalls.c:174-176`: `trap_AreasConnected(area1, area2)`
  → `syscall( G_AREAS_CONNECTED, area1, area2 )`. Plain ints both ways.
- **String** — `game/g_syscalls.c:114-116`: `trap_SendServerCommand(clientNum,
  text)` → `syscall( G_SEND_SERVER_COMMAND, clientNum, text )`. `const char*`
  passed raw — native DLL shares address space with engine, no translation.
- **Struct pointer (out-param)** — `game/g_syscalls.c:148-150`:
  `trap_Trace(trace_t *results, vec3_t start, mins, maxs, end, passEntityNum,
  contentmask)` → `syscall( G_TRACE, results, start, mins, maxs, end,
  passEntityNum, contentmask, 0, 10 )`. 5 pointer args + 2 hardcoded trailing
  ints (`0`, `10`, ghoul2 trace type/lod) not in the wrapper's own signature.
- **Float** — `game/g_syscalls.c:139-141`: `trap_SetServerCull(float
  cullDistance)` → `syscall(G_SET_SERVER_CULL, PASSFLOAT(cullDistance))`.

### 1.4 PASSFLOAT

`game/g_syscalls.c:21-25`:
```c
int PASSFLOAT( float x ) { float floatTemp; floatTemp = x; return *(int *)&floatTemp; }
```
Plain function (not a macro), local to `g_syscalls.c` (not `g_local.h`).
Bit-reinterprets a `float` as `int` so it fits one slot of the all-`int`
variadic `syscall` convention (some ABIs default-promote varargs `float`→
`double`, which would corrupt the fixed-width `int[]` reinterpretation in
`VM_DllSyscall`, §1.6). Reversed engine-side by `FloatAsInt`/`VMF()` (§1.5) —
same trick, opposite direction. Used pervasively for bot-AI floats, e.g.
`game/g_syscalls.c:628,746,834,842,846,854,869,1014,1044,1088,1096,1112,1259,1323,1341,1401`.

### 1.5 Engine-side dispatchers

All three share `args[0]` = syscall number (`int *args`), and `VMA`/`VMF` macro
pairs.

- **`server/sv_game.cpp:458`** — `int SV_GameSystemCalls( int *args )`. Macros
  `:400-404`: Linux/PPC → `#define VMA(x) ((void *) args[x])` (raw, no
  translation); else `#define VMA(x) VM_ArgPtr(args[x])` (`:403`). Float macro
  `:406`: `#define VMF(x) ((float *)args)[x]`. `FloatAsInt` helper `:384-390`.
  Examples: `G_MILLISECONDS` (`:509-510`, scalar return via `Sys_Milliseconds()`);
  `G_SEND_SERVER_COMMAND` (`:572-574`, string via `VMA(2)`); `G_TRACE`
  (`:587-588`, struct-out via `VMA(1)`); `G_SET_SERVER_CULL` (`:598-599`, float
  in via `VMF(1)`); `G_AREAS_CONNECTED` (`:627-628`, scalars).
- **`client/cl_cgame.cpp:644`** — `int CL_CgameSystemCalls( int *args )`. Macros
  `:625-626`: `VMA(x)` always → `VM_ArgPtr(args[x])` (no PPC/Linux raw branch
  here, unlike sv_game). `FloatAsInt` `:608-614`. Shares a `TRAP_*` block
  (100+, numbering shared across game/cgame/ui per comment `:643-645` /
  `sv_game.cpp:459-461`): `TRAP_MEMSET`/`TRAP_MEMCPY`/`TRAP_STRNCPY` (`VMA`),
  `TRAP_SIN`/`TRAP_COS`/`TRAP_SQRT` (`VMF` in, `FloatAsInt` out). Cgame-specific:
  `CG_MILLISECONDS` (scalar), `CG_PRINT` (`Com_Printf("%s", VMA(1))`, string in).
- **`client/cl_ui.cpp:813`** — `int CL_UISystemCalls( int *args )`. Same macros
  (`:802-803`), `VM_ArgPtr` forward-declared locally `:801`. Shares the
  `TRAP_*` block (`:816`+). UI examples: `UI_CVAR_REGISTER` (`:865-866`,
  `Cvar_Register((vmCvar_t*)VMA(1), (const char*)VMA(2), (const char*)VMA(3),
  args[4])` — struct-ptr + 2 strings + scalar); `UI_CVAR_VARIABLEVALUE`
  (`:874-875`, `FloatAsInt(Cvar_VariableValue((const char*)VMA(1)))`, float
  return); `UI_CVAR_SETVALUE` (`:881-882`, `Cvar_SetValue(VMA(1), VMF(2))`,
  float in).

**Native vs QVM pointer translation** — `VM_ArgPtr` (`qcommon/vm.cpp:640-654`,
full body verified):
```c
void *VM_ArgPtr( int intValue ) {
    if ( !intValue ) return NULL;
    if ( currentVM==NULL ) return NULL;
    if ( currentVM->entryPoint ) { return (void *)(currentVM->dataBase + intValue); }
    else { return (void *)(currentVM->dataBase + (intValue & currentVM->dataMask)); }
}
```
Note: even the native branch (`entryPoint` set) still does `dataBase +
intValue` — same offset-into-segment shape as QVM, differing only in the
`& dataMask` wrap. In practice native DLLs pass raw host pointers through
`syscall(...)` directly (§1.3), so `VM_ArgPtr` is bypassed outright on
Linux/PPC (`sv_game.cpp:401`, no `VM_ArgPtr` call) and is a pass-through no-op
elsewhere because `vm->dataBase` stays `NULL` for `VMI_NATIVE` vms (dataBase
allocation is skipped in the native early-return path, `qcommon/vm.cpp:509-524`).

### 1.6 VM layer (`qcommon/vm.cpp`)

- `vmInterpret_t` — `qcommon/qcommon.h:275-279`: `{ VMI_NATIVE, VMI_BYTECODE,
  VMI_COMPILED }`.
- **`VM_Create`** — `qcommon/vm.cpp:471-472`: `vm_t *VM_Create(const char
  *module, int (*systemCalls)(int *), vmInterpret_t interpret)`. Native path
  (`:509,515-524`): `Sys_LoadDll(module, &vm->entryPoint, VM_DllSyscall)`; on
  success **returns immediately** — skips all QVM setup (no `.qvm` read, no
  header validate/byteswap, no `dataBase`/`dataMask` alloc, no
  `VM_Compile`/`VM_PrepareInterpreter`, `:527-583`, only reached on native-load
  failure). `fs_restrict` cvar forces `VMI_NATIVE → VMI_COMPILED` even when
  native requested (`:509-513`).
- **`Sys_LoadDll`** — `win32/win_main.cpp:811-812`. Resolves `dllEntry`
  (`:879`) + `vmMain` (`:880`) via `GetProcAddress`, then calls `dllEntry(
  systemcalls )` (`:885`) — this hands the module its syscall trampoline
  (lands at `game/g_syscalls.c:14`).
- **`VM_Call`** — `qcommon/vm.cpp:787`: `int QDECL VM_Call( vm_t *vm, int
  callnum, ... )`. Local `int args[16]` (`:791`); native path (`:809-819`)
  forwards **16 words** via `vm->entryPoint(callnum, args[0..15])` — more than
  the 12-arg `vmMain` signature consumes; extras are silently dropped by the
  callee's fixed parameter list (matches `VM_DllSyscall`'s own comment
  admission that it "just grab[s] 15 arguments... the extra is thrown away",
  `qcommon/vm.cpp:355-357`). Non-native dispatches to
  `VM_CallCompiled`/`VM_CallInterpreted` (`:820-823`) — bytecode path, entirely
  skipped for native DLLs.
- **`VM_ArgPtr`** — `qcommon/vm.cpp:640-654` (quoted above).
- **`VM_DllSyscall`** — `qcommon/vm.cpp:363-380`. Non-PPC/Linux (`:378-379`):
  `return currentVM->systemCall( &arg );` — treats `&arg`'s address as base of
  a contiguous `int[]`, relying on x86 cdecl stack contiguity (comment block
  `:326-359` calls this "The horror; the horror", flags it platform-fragile).
  PPC/Linux (`:364-376`) instead copies varargs into `int args[16]` via
  `va_arg` before calling `systemCall(args)`.

---

## 2. SP table ABI

### 2.1 `game_import_t` / `game_export_t` — table, not syscall-numbered

`code/game/g_public.h:168-471` (`game_import_t`) / `:476-527` (`game_export_t`).
Both are plain structs of literal C function-pointer members — no integer
syscall-number indirection at all. `GAME_API_VERSION = 8` (`g_public.h:5`).

`game_import_t` sample (engine→game): `Printf` (`:172`), `Error` (`:179`),
`Milliseconds` (`:184`), `cvar` (`:187`), `FS_FOpenFile` (`:196`),
`SetConfigstring` (`:240`), `trace` (`:259-260`), `linkentity` (`:276`),
`Malloc` (`:290`), `G2API_PrecacheGhoul2Model` (`:297`) — Ghoul2 block runs
`:294-467`, dozens of `G2API_*`/`WE_*` fields (e.g. `WE_GetWindVector` `:456`).

`game_export_t` sample (game→engine), `:476-527`: `apiversion` (`:477`, plain
int — version handshake, not a fn ptr), `Init` (`:482`), `Shutdown` (`:483`),
`WriteLevel`/`ReadLevel` (`:487-488`), `ClientConnect` (`:493`), `ClientThink`
(`:499`), `RunFrame` (`:501`), `ConsoleCommand` (`:508`), and raw data fields
`gentities`/`gentitySize`/`num_entities` (`:524-526`) — the shared-array-handoff
idiom, SP's analog of `trap_LocateGameData`. Entrypoint prototype:
`game_export_t *GetGameApi(game_import_t *import);` (`:529`).

**Contrast**: no numbered trap table anywhere — every cross-boundary call is a
directly-typed fn-ptr struct field populated once at load. This is a
Quake2-style "struct of pointers" ABI, not MP's `args[0]`-indexed convention.

### 2.2 `GetGameAPI` entrypoint + version check

`code/game/g_main.cpp:875`: `game_export_t *GetGameAPI( game_import_t *import
) {`. Body (`:876-916`): `gi = *import;` (`:879`, copies whole import struct by
value into game-side global), `globals.apiversion = GAME_API_VERSION;`
(`:880`), per-field assignment e.g. `globals.Init = InitGame;` (`:881`),
`globals.Shutdown = ShutdownGame;` (`:882`), `globals.ClientThink =
ClientThink;` (`:888`), `globals.RunFrame = G_RunFrame;` (`:891`),
`globals.ConsoleCommand = ConsoleCommand;` (`:894`), `globals.gentitySize =
sizeof(gentity_t);` (`:898`); `return &globals;` (`:916`, static-scoped struct,
not heap, one instance).

Version check is **engine-side**, not inside `GetGameAPI`:
`code/server/sv_game.cpp:680-682`:
```c
if (ge->apiversion != GAME_API_VERSION)
    Com_Error (ERR_DROP, "game is version %i, not %i", ge->apiversion, GAME_API_VERSION);
```
immediately after `ge = (game_export_t *)Sys_GetGameAPI (&import);` (`:667`).

### 2.3 `code/client/vmachine.cpp` — SP's cgame VM shim (native, no bytecode)

Exists (39 lines), covers **cgame only** (not the game module — see 2.2).

- `VM_Call` — `client/vmachine.cpp:12-24`: `int VM_Call( int callnum, ... ) {
  if (cgvm.entryPoint) return cgvm.entryPoint( (&callnum)[0], ...,
  (&callnum)[9] ); return -1; }`. Forwards 10 words via pointer arithmetic off
  `&callnum` (not variadic iteration) into `cgvm.entryPoint`, a bare `int
  (*)(int, ...)`. Direct native call through a fn pointer — no opcode
  fetch/dispatch loop in this file.
- `VM_DllSyscall` — `client/vmachine.cpp:36-39`: `return
  CL_CgameSystemCalls( &arg );` (cgame calling back into client) — again a
  purely native shim shaped to look like a VM syscall.
- `vm_t` — `client/vmachine.h:48-52`: `struct vm_s { int (*entryPoint)( int
  callNum, ... ); }` — raw fn pointer only, no `vmHeader_t`, no bytecode
  interpreter loop anywhere in `code/`. **SP never runs a QVM interpreter.**
- `VM_Create` (`client/vmachine.h:72-91`) only recognizes module `"cl"`
  (`:76`); anything else returns `0` (`:87`). For `"cl"`: `Sys_LoadCgame(
  &cgvm.entryPoint, VM_DllSyscall )` (`:78`).
- `Sys_LoadCgame` (real loader) — `code/win32/win_main.cpp:557-570`:
  `GetProcAddress(game_library, "dllEntry"/"vmMain")` then `dllEntry(
  systemcalls )`. `game_library` (`win32/win_main.cpp:459`) is the **same**
  HINSTANCE already loaded for the game module via `Sys_GetGameAPI`
  (`:483-546`, `LoadLibrary(...jagamex86.dll...)` `:516/525`,
  `GetProcAddress(game_library, "GetGameAPI")` `:540`) — confirmed by
  `game.vcproj` listing both `g_main.cpp` and `cgame/cg_main.cpp` as sources of
  one `jagamex86.dll` target. Engine call sequence,
  `code/server/sv_game.cpp:667-679`: `ge = Sys_GetGameAPI(&import);` (table)
  then `VM_Create("cl")` (callnum, same DLL).

### 2.4 `code/cgame/cg_main.cpp` — SP cgame entrypoint

`int vmMain( int command, int arg0..arg7 ) {` — `cg_main.cpp:94`, dispatches
on `cgameExport_t` (`client/vmachine.h:13-39`), e.g. `CG_INIT` (`:98-100`),
`CG_DRAW_ACTIVE_FRAME` (`:107-108`, `CG_DrawActiveFrame(arg0,
(stereoFrame_t)arg1)`). Invoked from `CL_InitCGame`
(`code/client/cl_cgame.cpp:1029-1047`), specifically `VM_Call( CG_INIT,
clc.serverCommandSequence );` (`:1047`) — through the `cgvm.entryPoint`/
`VM_Call` shim (2.3), not a plain C call and not the `game_import_t` table
pattern. Compiled into `game.vcproj` alongside `g_main.cpp` → **statically
linked into `jagamex86.dll`** (the game DLL), not into the engine exe.
`starwars.vcproj` (the engine exe) contains `client/vmachine.cpp` but not
`cg_main.cpp`/`g_main.cpp`.

This refines (doesn't contradict) `CLAUDE.md`'s "SP cgame/ui are statically
linked into the engine binary": cgame is statically linked into the **game
DLL**, which the engine exe (`starwars.exe`) loads dynamically.

### 2.5 UI — third distinct shape: static link into exe, table-as-argument

`ui/ui_main.cpp` compiles directly into `starwars.vcproj`/`x_exe.vcproj` (the
exe), not `game.vcproj`. `client/cl_ui.cpp` calls UI as a **plain linked C
function**, not via `uivm.entryPoint`/`VM_Call`:
```c
void UI_Init( int apiVersion, uiimport_t *uiimport, qboolean inGameLoad );  // cl_ui.cpp:193
UI_Init(UI_API_VERSION, &uii, ...);  // cl_ui.cpp:297
```
`vm_t uivm;` declared (`cl_ui.cpp:362`) but dead for this path — no
`VM_Create`/`VM_Call` for `"ui"` anywhere (`vmachine.h:76,87` only special-cases
`"cl"`). UI still gets an import table, `uiimport_t` (`code/ui/ui_public.h`,
closes `:141`), passed **as an argument**, not returned from a factory.
Callee-side version check (inverted vs sv_game's caller-side check):
`code/ui/ui_atoms.cpp:248-249`:
```c
if ( apiVersion != UI_API_VERSION )
    ui.Error( ERR_FATAL, "Bad UI_API_VERSION: expected %i, got %i\n", UI_API_VERSION, apiVersion );
```
`UI_API_VERSION = 3` (`ui/ui_public.h:8`).

**Three distinct SP seam shapes**: (a) game — dynamic DLL, table
(`game_import_t`/`game_export_t`), factory `GetGameAPI`, engine-side version
check; (b) cgame — static-linked into the *same* game DLL, callnum-dispatch
`vmMain`/`VM_Call` shim resembling MP but zero bytecode, all native; (c) UI —
static-linked into the exe itself, plain C calls with import-table argument,
callee-side version check.

---

## 3. Current Rust ABI-seam state

### 3.1 `crates/abi-transport` (10 files, 318 lines) — transport primitives

Workspace member `crates/abi-transport` (`Cargo.toml:10`), crate name
`abi_transport`.

- `lib.rs` (20 lines) — doc: "cross-mode ABI transport: syscall/vmMain word
  packing, `OutboundSysCall`/`InboundVmCall` traits, function-table shapes, raw
  `dllEntry`/`vmMain`/`GetGameAPI` entrypoint types, and Raven `PASSFLOAT`. No
  Raven game types cross here." `pass_float` (`:17-19`): `pub fn
  pass_float(f: f32) -> isize { f.to_bits() as i32 as isize }`.
- `entrypoints.rs` (76 lines, verified in full) — raw C-ABI aliases
  (`RawSyscall = *const c_void`, `RawVmMain` = 12-word extern "C" fn type
  matching §1.1 exactly) + **stub** exports:
  - `qvm::dllEntry(_syscall: RawSyscall) {}` — discards the pointer, does not
    store it anywhere.
  - `qvm::vmMain(_command, _arg0..._arg11) -> AbiWord { 0 }` — ignores every
    arg, unconditionally returns 0.
  - `qvm::GetModuleAPI(...) -> RawExportTable { core::ptr::null_mut() }`.
  - `sp_game::GetGameAPI(_import) -> RawExportTable { core::ptr::null_mut() }`.
  None of these carry a `//TODO: Port` marker (undocumented no-ops, not yet
  normalized to the porting-rules marker convention).
- `generic/mod.rs`, `generic/message.rs`, `generic/inbound.rs`,
  `generic/outbound.rs`, `generic/table.rs`, `generic/transport/{mod,syscall,
  vm_main}.rs` — trait/type layer:
  ```rust
  // generic/outbound.rs:5-11
  pub trait OutboundSysCall { type Import; type Args; type Output; const IMPORT: Self::Import; }
  // generic/inbound.rs:4-10
  pub trait InboundVmCall { type Command; type Args; type Output; const COMMAND: Self::Command; }
  // generic/transport/syscall.rs:29-40
  pub trait EncodeSysCall: OutboundSysCall { fn encode_syscall(args: &Self::Args) -> SysCallTransport; }
  pub trait DecodeSysCallReturn: OutboundSysCall { fn decode_return(word: isize) -> Self::Output; }
  // generic/transport/vm_main.rs:29-40
  pub trait DecodeVmMain: InboundVmCall { fn decode_vm_main(transport: VmMainTransport) -> Self::Args; }
  pub trait EncodeVmMainReturn: InboundVmCall { fn encode_return(output: Self::Output) -> isize; }
  ```
  `generic/table.rs` — `FunctionTableImport`/`FunctionTableExport` traits for
  the SP-game table ABI (§2.1).

**Stored syscall pointer**: none. `RawSyscall` is discarded by the stub
`dllEntry`. No struct anywhere (grepped) holds it for later dispatch.

**Decode→dispatch→encode round trip**: does not exist. 885 files across
`crates/mp/abi` + `crates/sp/abi` implement `EncodeSysCall`/
`DecodeSysCallReturn` (typed per-syscall halves); 34 implement
`DecodeVmMain`/`EncodeVmMainReturn`. But `impl ... OutboundSysCallExecutor for`
/ `impl ... InboundVmCallExecutor for` has **zero** concrete implementations
anywhere — only the blanket `impl<T> MessageOutboundSysCallExecutor for T
where T: OutboundSysCallExecutor {}` (`generic/message.rs:44`), which itself
needs an executor impl that doesn't exist. No `dispatch` code, no `todo!("Port
...")` guarding the gap — it's simply absent scaffolding behind stub
entrypoints that silently no-op.

### 3.2 `crates/mp/abi` organization

```
game/{imports.rs, exports.rs, mod.rs}
game/syscalls/   330 files (G_*, BOTLIB_* outbound Args/Output)
game/vmcalls/    41 files  (GAME_* inbound Args/Output)
cgame/{imports.rs, exports.rs, mod.rs, public/}
cgame/syscalls/  234 files
cgame/vmcalls/   33 files
cgame/public/    12 files
ui/{imports.rs, exports.rs, mod.rs}
ui/syscalls/     151 files
ui/vmcalls/      13 files
ui/public/       5 files
```

Three samples:

1. **Plain scalar** — `crates/mp/abi/src/game/syscalls/G_MILLISECONDS.rs`:
   `GMillisecondsArgs` is a unit struct; `EncodeSysCall::encode_syscall`
   returns `SysCallTransport::new([] as [isize; 0])`; `DecodeSysCallReturn`
   returns `word as c_int`.
2. **Struct pointer** — `crates/mp/abi/src/game/syscalls/G_TRACE.rs:11-27,
   84-112`: `GTraceArgs { results: *mut trace_t, start/mins/maxs/end: *const
   vec3_t, pass_entity_num: c_int, contentmask: c_int }`. Represented as raw
   `*mut`/`*const` fields (not embedded by value, since the engine writes
   through `results`); `encode_syscall` calls `ptr_to_word` on each pointer and
   hardcodes the trailing `0, 10` (ghoul2 trace type/lod) matching
   `trap_Trace`'s own hardcoded args (§1.3). No `offset_of!`/`size_of` asserts
   on the Args carrier itself (those live on the pointee, e.g. `trace_t`).
3. **Float field** — `crates/mp/abi/src/game/syscalls/G_SIN.rs`: `GSinArgs {
   angle: f32 }`; `encode_syscall` → `SysCallTransport::new([pass_float(a.angle)])`;
   `decode_return` → `f32::from_bits(word as i32 as u32)`.

**Syscall-number enums**: `MpGameImport` (`crates/mp/abi/src/game/imports.rs:
8-1105`, ~330 variants, `#[repr(i32)]`) — explicit gap-preserving
discriminants at family boundaries (`G_MEMSET = 100` `:342`, `G_NAV_INIT = 200`
`:392`, `BOTLIB_SETUP = 250` `:520`, `BOTLIB_AAS_ENABLE_ROUTING_AREA = 300`
`:560`, `BOTLIB_EA_SAY = 400` `:617`, `BOTLIB_AI_LOAD_CHARACTER = 500` `:695`).
File doc (`:1-5`): "These discriminants are ABI wire values; do not renumber
them." Inbound counterpart `MpGameExport` (`exports.rs:9`, 40 `GAME_*`
variants). Parallel per-module: `cgame/{imports,exports}.rs` (32 `CG_*`),
`ui/{imports,exports}.rs` (12 `UI*`).

### 3.3 `crates/sp/abi` organization (table half)

```
lib.rs
game/{imports.rs, exports.rs, mod.rs, public/}
game/public/  game_export_t.rs, game_import_t.rs, saved_game_just_loaded_e.rs
cgame/{mod.rs, public/, syscalls/(123 files), vmcalls/(18 files)}
ui/{mod.rs, public/, syscalls/(96 files), vmcalls/(1 file)}
```

SP cgame/ui still use the syscall/vmMain word shape (matching §2.3-2.5's
native-but-vmMain-shaped reality); only SP game uses the table ABI, per
`crates/sp/abi/src/game/mod.rs:1-4`: "Raven SP game uses the `GetGameAPI`
function-table ABI, not the `vmMain`/syscall shape. Keep that surface deferred
until the table ABI is modeled."

**Two unconnected layers for the same Raven type**:

1. Trait-token + opaque-table (`game/imports.rs:1-27`, `exports.rs:1-27`):
   ```rust
   #[repr(C)]
   pub struct SpGameImportTable { _private: [u8; 0] }   // "layout intentionally deferred"
   pub struct SpGameImport;
   impl FunctionTableImport for SpGameImport { type Table = SpGameImportTable; }
   ```
   (mirrored `SpGameExport`/`SpGameExportTable` in `exports.rs`).
2. Fully-laid-out struct (`game/public/game_import_t.rs`, `game_export_t.rs`)
   — actual Raven struct, `#[repr(C)]`, every field `Option<unsafe extern "C"
   fn(...)>`:
   ```rust
   // game_import_t.rs:26-33
   #[repr(C)]
   pub struct game_import_t {
       pub Printf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
       pub WriteCam: Option<unsafe extern "C" fn(text: *const c_char)>,
       ...
   }
   #[cfg(target_pointer_width = "64")]
   const _: () = assert!(core::mem::size_of::<game_import_t>() == 1048);   // :596
   const _: () = assert!(core::mem::offset_of!(game_import_t, Printf) == 0); // :598
   ```
   `game_export_t` similarly (144 bytes asserted `:101`), including trailing
   `gentities: *mut gentity_t`, `gentitySize`, `num_entities` data fields
   (Raven's shared-array idiom, §2.2).

These two layers are **not wired together** — `SpGameImportTable`/
`SpGameExportTable` stay `_private: [u8; 0]` opaque despite the full structs
existing one directory over; grep confirms no cross-reference.
`crates/jagame/src/lib.rs:1` (the SP jagame crate) only re-exports the generic
stub `GetGameAPI` from `abi-transport` (returns `null_mut()`), so the
fully-typed `game_export_t` is never actually returned live.

**Structural comparison**: MP/SP-cgame/SP-ui model the seam as many small
typed call objects (numbered enum + word-args, encode/decode per call); SP
game models it as one big `#[repr(C)]` fn-pointer struct exchanged once via
`GetGameAPI` — a single aggregate vs. N discrete numbered messages. SP game's
struct layout is already correct/asserted but disconnected from any live
entrypoint.

---

## 4. Legacy design: `docs/engine-plan.md` (123 lines) vs current crates

| # | Mechanism (engine-plan.md) | Status against current crates |
|---|---|---|
| 1 | Split transport into exists/add halves: `EncodeSysCall`+`DecodeSysCallReturn` (out), `DecodeVmMain`+`EncodeVmMainReturn` (in) (`:35-39`) | **Shipped, current reality.** All 4 traits live in `crates/abi-transport/src/generic/transport/{syscall,vm_main}.rs`, implemented 885×/34× (§3.1-3.2). |
| 2 | `Execute<C: OutboundSysCall> { fn execute(&self, args: C::Args) -> C::Output }`, backend-blanket-implemented (`:41-44`) | **Does not exist.** No `Execute<` anywhere. `OutboundSysCallExecutor`/`InboundVmCallExecutor` are the closest analog (abi-transport) but have zero concrete impls. |
| 3 | `CEngine` backend: blanket `impl<C> Execute<C> for CEngine where C: EncodeSysCall+DecodeSysCallReturn`, calling `unsafe { raw_syscall_words(...) }` as "sole unsafe choke point" (`:46-58`) | **Does not exist.** No `struct CEngine`, no `raw_syscall_words`. This is exactly the missing dispatch piece — the `RawSyscall` pointer `CEngine` would hold is discarded by the stub `dllEntry` (§3.1). |
| 4 | `RustEngine` backend + `RunNative` trait, per-call handlers (`:63-77`) | **Does not exist.** No engine-state struct of any kind behind the seam. |
| 5 | `trap::X(..)` call-site syntax + `type Engine = CEngine` backend-select alias (`:79-88`) | **Does not exist.** No `trap::` module, no `static ENGINE`/`struct Engine`. The 885 syscall types are pure data defs, never invoked from a wrapper. |
| 6 | Inbound `vm_main` switch via `DecodeVmMain`/`EncodeVmMainReturn` instead of hand-indexing `args[]` + hand-returning 0 (`:90-93`) | **Earlier-stage than the doc's "before" state.** Current `qvm::vmMain` stub ignores `args[]` and returns 0 unconditionally — but isn't even a switch; there's no per-command branching at all. |
| 7 | Path framing: `src/engine`, `src/abi/` (`:4`) | **Stale.** Repo has migrated (this is the `crate-migration` branch) to `crates/{mp,sp}/{engine,abi}`, `crates/abi-transport`; no `src/` dir exists at repo root. |
| 8 | Incremental steps 1-8 (`:102-105`+) | Step 1 done (halves, see row 1). Steps 2-8 (Execute/CEngine, tracer-bullet `Cvar_Register`, `GAME_RUN_FRAME` routing, `RustEngine` skeleton, global consolidation) all unstarted. |

**Conclusion**: engine-plan.md's transport-layer half (traits 1) is now
baseline reality and has been extended far past the doc's ambition (885+34
typed Args/Output vs. the doc's incremental step-6 aspiration). Everything
about *execution* — `Execute`, `CEngine`, `RustEngine`, `raw_syscall_words`,
`trap::`, an `ENGINE` accessor — is wholly unbuilt. Net effect: a large
inventory of fully-encoded/decoded call types with no executor to invoke them
and no entrypoint that does anything but return a constant/null.

---

## 5. WASM-relevant constraints

### 5.1 DEC-05 (`docs/decisions.md:49-70`)

Full entry `:49-70`; WASM sub-clause verbatim, `:63-70`:

> 5. **WASM module transport: first-class target from the start.** The module
>    transport is pluggable — `NativeDll | Static | Wasm` — and
>    `architecture/engine-seam.md` + `architecture/module-loading.md` must
>    design the WASM variant explicitly (wasm32 linear-memory pointer
>    translation à la `VM_ArgPtr`, handle-only trap surface, 32-bit in-module
>    layouts). Module crates get `wasm32` build checks in CI from the beginning
>    so portability never regresses. The wasmtime host itself lands **after**
>    native-DLL parity is proven (a sandbox is no place to debug parity).

Other DEC-05 items (`:51-62`): (1) Rust engine ↔ Rust modules, all platforms,
via `native/platform` dylib loading honoring Raven entry symbols; (2) Rust
modules loaded by real engines (retail jamp 1.01, `i686-pc-windows`; OpenJK,
account for ABI divergences per `tools/closure-prototype/NOTES.md`); (3) Rust
engine hosting real/mod DLLs (JA+, MBII) — `i686-pc-windows` engine build only;
(4) classic QVM bytecode interpreter explicitly **out of scope**.

**Constraints for the design session**: transport must be enum-dispatched
pluggable from day one, not retrofitted; must explicitly solve pointer
translation with `VM_ArgPtr` named as precedent (§1.6); WASM trap surface must
degrade to handle-only semantics; `wasm32` is a standing CI gate; wasmtime host
work is sequenced after native-DLL parity (parity is proven against a real
sandbox-free target first).

### 5.2 `docs/abi-traps.md` trap categorization (313 traps total)

Source: `docs/abi-traps.md:1-9` header, generated from
`oracle/oracle/codemp/game/g_syscalls.c`. File's own heuristic column splits:
scalar 134, string 64, ptr 40, opaque 75. Remapped into 5 transport-relevant
buckets (counts are order-of-magnitude; some traps are borderline):

| Bucket | Count (approx) | Representative traps (docs/abi-traps.md line) |
|---|---|---|
| Scalar-only | ~134 | `trap_Milliseconds` (:12), `trap_PointContents` (:39), `trap_AreasConnected` (:43), `trap_Characteristic_Float` (:193), `trap_EA_Attack` + ~20 sibling `trap_EA_*` (:171) |
| String in/out | ~64 | `trap_Printf` (:10), `trap_Cvar_Set` (:17), `trap_Cvar_VariableStringBuffer` (:19), `trap_GetConfigstring` (:31), `trap_FS_GetFileList` (:54) |
| Struct in/out (fixed-size, single-call) | ~46 | Named: `trap_Cvar_Update`(:16, `vmCvar_t*`), `trap_Trace`/`trap_G2Trace`/`trap_TraceCapsule`(:37,38,59, `trace_t*`), `trap_GetUsercmd`/`trap_BotUserCommand`(:50,142, `usercmd_t*`), `trap_RealTime`(:57, `qtime_t*`), `trap_SiegePersSet`/`Get`(:52-53), `trap_PC_ReadToken`(:268), `trap_DebugPolygonCreate`(:55, `vec3_t*` array). Anonymous-via-`void*`: `trap_AAS_EntityInfo`(:143), `trap_AAS_PredictClientMovement`(:164), `trap_BotPushGoal`/`trap_BotGetTopGoal`(:221,227), `trap_BotMoveToGoal`(:249), `trap_BotGetWeaponInfo`(:260) |
| Entity-array / shared-memory | ~19 | `trap_LocateGameData`(:27, registration), `trap_AdjustAreaPortalState`/`LinkEntity`/`UnlinkEntity`/`EntityContact`/`EntityContactCapsule`(:42,44,45,47,60), `trap_ICARUS_ValidEnt`/`TaskIDPending`/`InitEnt`/`FreeEnt`/`AssociateEnt`/`TaskIDSet`/`TaskIDComplete`(:72,76-79,81-82), `trap_Nav_GetNearestNode`/`CheckFailedNodes`/`AddFailedNode`/`NodeFailed`/`GetBestPathBetweenEnts`(:98,108-110,121) |
| Function-pointer / handle | ~50 | Ghoul2 opaque-handle family (~47 rows): `trap_G2API_InitGhoul2Model`(:278, `void **ghoul2Ptr` handle written back), `trap_G2API_GetBoltMatrix`(:275), `trap_G2API_SetBoneAngles`(:283), `trap_G2API_CollisionDetect`(:294), `trap_G2API_DuplicateGhoul2Instance`(:289, `void *g2From, void **g2To`); non-Ghoul2: `trap_PrecisionTimer_Start`/`End`(:13-14), `trap_TrueMalloc`/`TrueFree`(:67-68) |

Sums to ~313, matching the table's row count (some traps debatably
reclassifiable between struct/opaque or entity-array/struct).

Footnote: `trap_SV_RegisterSharedMemory`/`trap_CG_RegisterSharedMemory`
(declared `oracle/oracle/codemp/game/g_local.h:1976`,
`oracle/oracle/codemp/cgame/cg_local.h:2421`; called once at init with
`gSharedBuffer`/`cg.sharedBuffer`, `oracle/oracle/codemp/game/g_main.c:920`,
`oracle/oracle/codemp/cgame/cg_main.c:3713`) is the same *pattern* as
`LocateGameData` but no consuming engine-side handler was found in
`sv_game.cpp` — likely dead/no-op in MP; worth a design footnote, not a
load-bearing citation.

### 5.3 `trap_LocateGameData` deep dive — the hard WASM case

Game-side wrapper — `oracle/oracle/codemp/game/g_syscalls.c:105-108`:
```c
void trap_LocateGameData( gentity_t *gEnts, int numGEntities, int sizeofGEntity_t,
                         playerState_t *clients, int sizeofGClient ) {
    syscall( G_LOCATE_GAME_DATA, gEnts, numGEntities, sizeofGEntity_t, clients, sizeofGClient );
}
```
Engine-side handler (verified) — `oracle/oracle/codemp/server/sv_game.cpp:
327-335`:
```c
void SV_LocateGameData( sharedEntity_t *gEnts, int numGEntities, int sizeofGEntity_t,
                       playerState_t *clients, int sizeofGameClient ) {
    sv.gentities = gEnts;
    sv.gentitySize = sizeofGEntity_t;
    sv.num_entities = numGEntities;
    sv.gameClients = clients;
    sv.gameClientSize = sizeofGameClient;
}
```
Dispatched via `VMA(1)`/`VMA(4)` — `sv_game.cpp:567`. Called **once** at
game-module init; engine keeps the raw pointers indefinitely in `sv.gentities`/
`sv.gameClients`.

Every-frame direct dereference, no per-call copy: `SV_GentityNum`
(`sv_game.cpp:54-58`, `(byte*)sv.gentities + sv.gentitySize*num`),
`SV_GameClientNum` (`:62-65`), inverse `SV_NumForGentity` (`:46-49`, `((byte*)ent
- (byte*)sv.gentities) / sv.gentitySize` — this is how *other* traps' bare
`gentity_t*` args resolve back to an index, confirming bucket-4 traps all
reference offsets into this one blob). Called dozens of times/frame outside
any trap: `sv_snapshot.cpp:339,542,551,601`, `sv_world.cpp:475,535,543,552,889`,
`sv_bot.cpp:193`, `sv_client.cpp:507,957`, `sv_init.cpp:214,717`,
`sv_main.cpp:358,701`.

**Why hardest for WASM**:
1. Native DLL: zero-copy by construction — one address space, `sv.gentities`
   genuinely aliases live module memory.
2. WASM: `gEnts`/`clients` as seen by the module are i32 offsets into that
   module's **private linear memory** — the host cannot walk
   `sv.gentities + stride*num` as a native pointer; there's no shared address
   space.
3. DEC-05 names the fix precedent: `VM_ArgPtr` (`qcommon/vm.cpp:640-654`,
   §1.6) *is* the "offset→host pointer" translation, just for the QVM sandbox.
   A wasmtime transport must reimplement the equivalent using the module
   `Memory`'s `data_ptr()`/`data()` (or `memory.read`/`memory.write`),
   re-deriving `(base+offset, len)` **per access**, not once.
4. The one-shot-registration idiom doesn't survive `memory.grow` — a resize
   after registration can move/invalidate a cached `data_ptr()` base. A
   faithful WASM port cannot cache a raw pointer like `sv.gentities` does; must
   re-resolve per access, or move to explicit handle+read/write calls (what
   DEC-05's "handle-only trap surface" is steering toward).
5. Net: `NativeDll`/`Static` backends implement bucket-4 traps as literal
   pointer storage + arithmetic (1:1 oracle parity, zero overhead); `Wasm`
   backend needs the same trap *signature* but a translation layer underneath
   — bounds-checked, revalidated every access, strictly slower. This is
   exactly the seam DEC-05 requires be designed explicitly, not discovered
   late.

---

## Design forks

Genuine open choices for the design session, each with its constraint evidence:

1. **Where does the transport trait live, and what's its shape?**
   Evidence: `Execute<C: OutboundSysCall>` (engine-plan.md:41-44, §4 row 2) is
   the only prior proposal and is unbuilt; current `abi-transport` only has
   the encode/decode halves (§3.1) plus unused `*Executor` marker traits with
   zero impls. Fork: single `Execute<C>`-style trait genericized over
   backend, vs. per-backend concrete dispatcher, vs. an enum-dispatched
   `Transport { NativeDll, Static, Wasm }` per DEC-05's literal wording
   (§5.1) — the DEC-05 phrasing implies an enum/runtime-selectable value, not
   just a generic type parameter; these have different implications for
   whether backend selection is compile-time (`type Engine = CEngine`,
   engine-plan.md:88) or runtime (needed if one process must support multiple
   module transports simultaneously, e.g. some modules native DLL + some
   WASM in the same session).

2. **How does the SP table ABI (`game_import_t`/`game_export_t`) unify with
   the MP numbered-syscall trait model under one transport abstraction?**
   Evidence: §3.3 — two structurally different seam shapes already coexist
   (`FunctionTableImport`/`Export` marker traits vs. `OutboundSysCall`/
   `InboundVmCall` per-message traits) and are *not* wired together even
   within SP itself. A pluggable `NativeDll|Static|Wasm` transport (DEC-05)
   must decide whether the table shape is a special case of the same trait
   family, or a genuinely separate protocol the transport enum must branch on.

3. **Does the engine ever store the raw syscall pointer, and where?**
   Evidence: §3.1 — `dllEntry`'s `RawSyscall` is discarded today; engine-plan's
   `CEngine`/`raw_syscall_words` (§4 rows 2-3) is the only design that
   proposed an owner. Per porting-rules §B (state is threaded, not reached;
   one owned instance per singleton), this needs a concrete home — a
   `CEngine`-like struct threaded explicitly, not a static.

4. **How are the ~285 struct/entity-array/handle traps (buckets 3-5, §5.2,
   ~115 of 313) marshalled per-transport without hand-writing 115 bespoke
   codecs per backend?**
   Evidence: current `EncodeSysCall`/`DecodeSysCallReturn` impls (§3.2) are
   already hand-written per trap (885 files) but only handle the
   encode/decode *halves*, not backend-specific memory translation. For
   `NativeDll`, a `*mut trace_t` is already host-valid; for `Wasm`, the same
   `GTraceArgs.results: *mut trace_t` field must become a bounds-checked
   linear-memory read/write. Fork: push translation into a shared trait method
   parameterized by backend (one generic marshaller), vs. generate two
   codegen targets from one spec, vs. accept per-struct manual `Wasm` impls
   for the ~46 struct-in/out + ~19 entity-array traps as a bounded one-time
   cost.

5. **How is `trap_LocateGameData` (and the `SV_RegisterSharedMemory`
   footnote pattern) modeled per transport?**
   Evidence: §5.3 — native caches a raw pointer once; WASM cannot (memory.grow
   invalidates cached bases) and DEC-05 explicitly demands this be designed,
   not discovered late. Fork: (a) transport-level "shared region" abstraction
   returning an opaque handle + explicit accessor calls that native
   implements as pointer arithmetic and WASM implements as
   offset-into-`Memory` with per-access bounds checks; vs. (b) keep
   `LocateGameData`'s call shape faithful (raw pointer args) for native/static
   and give WASM a structurally different registration call — which breaks
   "same trap signature across backends" and complicates the dispatch-table
   generation story (fork 6).

6. **Are engine-side dispatch tables (`SV_GameSystemCalls`-equivalent, §1.5)
   hand-written per module or generated from the `MpGameImport`/
   `MpGameExport` enums (§3.2)?**
   Evidence: 330+41+234+33+151+13 = 802 MP-side Args/Output files already
   exist as a machine-checkable manifest (enum variant ↔ struct ↔
   encode/decode impl); engine-plan's `trap::X` call-site sugar (§4 row 5) and
   the missing `CEngine` (§4 row 3) both presuppose *some* mechanical
   generation path from that manifest rather than a hand-written switch per
   `SV_GameSystemCalls`-equivalent. Fork: proc-macro/build-script codegen
   keyed off the enum, vs. a hand-maintained dispatch match per backend (as
   Raven itself does, §1.5) accepting the duplication as the oracle-parity
   price of doing it exactly like Raven.

7. **Does `wasm32` CI-gating (DEC-05) apply to `crates/mp/abi`/`crates/sp/abi`
   as-is, or do they need a backend-parameterized split first?**
   Evidence: current Args/Output structs (§3.2 samples) hold raw
   `*mut T`/`*const T` fields directly (e.g. `GTraceArgs.results: *mut
   trace_t`) — those compile for `wasm32` today (pointers are valid wasm32
   types) but are *meaningless* as wasm32 linear-memory offsets without the
   translation layer from fork 4/5. Fork: is "builds for wasm32" (DEC-05's
   literal CI requirement) sufficient as an early gate, or must the Args
   shape itself change (e.g. `results: u32` offset newtype instead of `*mut
   trace_t`) before wasm32-correctness (not just wasm32-compiles) is
   achievable — the latter would ripple through all 802 MP + ~240 SP
   Args/Output files.
