# Engine Seam Design
Status: DRAFT     Supersedes: docs/engine-plan.md
Decision prefix: SEAM     Ledger deps: DEC-04, DEC-05, DEC-07, DEC-08, DEC-09

## Standing context

Links only — never restated here:

- `docs/workspace-architecture.md` — crate graph and tiers (`abi-transport`,
  `crates/{mp,sp}/{abi,engine}`, `crates/jagame`).
- `docs/porting-rules.md` — §B (state spine, the §B6 single-singleton exception),
  §D11 (unsafe confined to the seam), §D12 (`#[repr(C)]` layout parity), marker
  convention (`//TODO: Port <subject>`).
- `docs/decisions.md` — DEC-04 (per-mode duplication), DEC-05 (module transport
  `NativeDll | Static | Wasm`), DEC-07 (SP cgame/ui via the vmachine shim),
  DEC-08 (`Com_Error` = panic + `catch_unwind`), DEC-09 (verification layers).
- `docs/abi-traps.md` — the 313-trap categorization the WASM marshaller buckets.
- Sibling docs this one forward-references (pending): `docs/architecture/
  lifecycle.md`, `docs/architecture/module-loading.md`,
  `docs/architecture/state-ownership.md`, `docs/subsystems/*`.

## Scope & non-goals

This doc freezes **the typed ABI seam and its executors**: how module-side
outbound calls (`trap_*`/`gi.*`) execute against a backend, how engine-side
dispatchers receive and route `vmMain`/syscall/table traffic, the pluggable
module transport (`NativeDll | Static | Wasm`), and the host-side WASM
marshalling design (designed now, implemented post-parity per DEC-05.5).

Non-goals (punted, each with its owning doc):

- **Executable lifecycle / boot order** → `docs/architecture/lifecycle.md`.
- **Module loading, filenames, search paths, VM restart semantics** →
  `docs/architecture/module-loading.md`.
- **Full state-ownership spine beyond the seam singletons** (engine service
  traits, `level`/`g_entities`/`gclients` ownership) →
  `docs/architecture/state-ownership.md`.
- **Per-subsystem trap semantics** (what each `G_*`/`CG_*`/`UI_*` handler *does*)
  → `docs/subsystems/*`. This doc defines only how a trap *crosses*, not its body.

## Raven ground truth

### MP: the `int args[]` syscall convention (all three modules)

Every MP module exports `int vmMain( int command, int arg0..arg11 )` — a command
word plus twelve `int`-sized argument words, regardless of host word width, and
switches on its `GAME_*`/`CG_*`/`UI_*` command
(`oracle/oracle/codemp/game/g_main.c:515`,
`oracle/oracle/codemp/cgame/cg_main.c:190`,
`oracle/oracle/codemp/ui/ui_main.c:579`). UI additionally answers a version-
negotiation command `UI_GETAPIVERSION` (`ui/ui_main.c:581`). An **unrecognized
command word** is handled per module: game and UI fall through the `switch` to
`return -1` (`g_main.c:695`, `ui_main.c:624`), while cgame's `default` arm calls
`CG_Error( "vmMain: unknown command %i" )` (fatal) before its trailing
`return -1` (`cg_main.c:354-358`). Game and cgame both **escape the 12-word cap**
for high-arity calls by reading a shared-memory struct (`gSharedBuffer`; `C_Trace()`/
`C_G2Trace()` behind `CG_TRACE`/`CG_G2TRACE`, `cg_main.c:239,243`) instead of
`arg0..arg11`.

Outbound, the module holds one poisoned variadic function pointer
`static int (QDECL *syscall)( int arg, ... ) = ...-1;`
(`oracle/oracle/codemp/game/g_syscalls.c:8`), set exactly once by the engine at
load through `void dllEntry( int (QDECL *syscallptr)(...) ) { syscall = syscallptr; }`
(`g_syscalls.c:14-16`). Each `trap_*` wrapper packs its args and calls
`syscall(IMPORT, ...)`; the callee reinterprets the varargs region as a flat
`int[]`. Floats cross via bit-reinterpretation:
`int PASSFLOAT( float x ){ float t=x; return *(int*)&t; }`
(`g_syscalls.c:21-25`) — a plain function local to `g_syscalls.c`, needed because
some ABIs promote a variadic `float`→`double` and corrupt the fixed-width
`int[]`. Struct-out and string args are passed as raw host pointers, valid only
because a native DLL shares the engine's address space (`trap_Trace`,
`g_syscalls.c:148-150`; `trap_SendServerCommand`, `:114-116`).

Engine-side, each module's syscalls land in one dispatcher over `int *args`
(`args[0]` = syscall number): `SV_GameSystemCalls`
(`oracle/oracle/codemp/server/sv_game.cpp:458`), `CL_CgameSystemCalls`
(`client/cl_cgame.cpp:644`), `CL_UISystemCalls` (`client/cl_ui.cpp:813`). Each
reads pointer args through `VMA(x)` and float args through `VMF(x) =
((float*)args)[x]` (`sv_game.cpp:400-406`), inverting `PASSFLOAT` with
`FloatAsInt` on the way out (`sv_game.cpp:384-390`). `VMA` is either raw
(`(void*)args[x]`, Linux/PPC native) or `VM_ArgPtr(args[x])`:

```c
void *VM_ArgPtr( int intValue ) {          // qcommon/vm.cpp:640-654
    if ( !intValue || currentVM==NULL ) return NULL;
    if ( currentVM->entryPoint ) return (void *)(currentVM->dataBase + intValue);
    else return (void *)(currentVM->dataBase + (intValue & currentVM->dataMask));
}
```

For a native DLL `dataBase` stays `NULL` and this is a pass-through no-op; it is
the QVM offset→host-pointer translation, and DEC-05 names it the precedent the
WASM transport must reimplement per-access.

Inbound, the engine reaches the module through `VM_Call( vm_t *vm, int callnum,
... )` (`qcommon/vm.cpp:787`), which for a native DLL packs `int args[16]` and
forwards **sixteen** words to `vm->entryPoint` (`:808-819`) — four more than
`vmMain`'s twelve; the callee's fixed parameter list silently drops the extras.
Non-native paths dispatch to `VM_CallCompiled`/`VM_CallInterpreted` (`:820-823`),
out of scope (DEC-05.4). `VM_DllSyscall` (`vm.cpp:363-380`) is the module→engine
trampoline; its non-PPC branch `return currentVM->systemCall( &arg );` (`:378`)
treats `&arg` as the base of a contiguous `int[]` (the comment block calls it
"The horror; the horror", `:326-359`), while PPC/Linux copies varargs into
`int args[16]` first.

### SP: three seam shapes, no word-encoding layer

SP game uses a **struct-of-function-pointers** ABI, not numbered syscalls:
`game_import_t` / `game_export_t` are plain structs of typed C fn-pointer members
(`oracle/oracle/code/game/g_public.h:168-527`), `GAME_API_VERSION = 8`
(`g_public.h:5`), factory `game_export_t *GetGameApi( game_import_t *import );`
(`g_public.h:529`). The definition copies the import struct by value
(`gi = *import;`), fills each export field, sets `globals.gentitySize =
sizeof(gentity_t)`, and returns a static-scoped struct
(`code/game/g_main.cpp:875-916`). `game_export_t` carries the shared-array
handoff fields `gentities`/`gentitySize`/`num_entities` (`g_public.h:524-526`) —
SP's analog of `trap_LocateGameData`. The version check is **engine-side, caller
position**: after `ge = Sys_GetGameAPI(&import);` the engine compares
`ge->apiversion != GAME_API_VERSION` (`server/sv_game.cpp:680-682`).

SP cgame is statically compiled into the *same game DLL* but reached through a
fake-VM shim: `int VM_Call( int callnum, ... )` forwards ten words via pointer
arithmetic off `&callnum` into `cgvm.entryPoint` (a bare `int(*)(int,...)`,
`code/client/vmachine.cpp:12-24`), and `int VM_DllSyscall( int arg, ... ) {
return CL_CgameSystemCalls( &arg ); }` (`vmachine.cpp:36-39`) routes callbacks —
a callnum-dispatch `vmMain` shape with zero bytecode. DEC-07 keeps this shim as a
thin dispatch layer.

SP UI is statically linked into the exe and called as a **plain linked C
function** with its import table passed *as an argument*, not returned from a
factory: `UI_Init( UI_API_VERSION, &uii, ... )` (`code/client/cl_ui.cpp:297`),
`UI_API_VERSION = 3` (`ui/ui_public.h:8`). Its version check is **callee-side**:
`if ( apiVersion != UI_API_VERSION ) ui.Error(...)`
(`code/ui/ui_atoms.cpp:248-249`).

### `LocateGameData`: the one-shot pointer-registration idiom

`trap_LocateGameData(gEnts, numGEntities, sizeofGEntity_t, clients,
sizeofGClient)` (`g_syscalls.c:105-108`) is called **once** at game init;
`SV_LocateGameData` stores the raw pointers and strides straight into server
state — `sv.gentities`, `sv.gentitySize`, `sv.num_entities`, `sv.gameClients`,
`sv.gameClientSize` (`sv_game.cpp:327-335`, dispatched via `VMA(1)`/`VMA(4)` at
`:567`). The engine then reads that blob dozens of times per frame *outside any
trap*, by stride arithmetic — `SV_GentityNum` = `(byte*)sv.gentities +
sv.gentitySize*num` (`sv_game.cpp:54-58`), `SV_GameClientNum` (`:62-65`), and the
inverse `SV_NumForGentity` = `((byte*)ent - (byte*)sv.gentities) /
sv.gentitySize` (`:46-49`), which is how every other trap's bare `gentity_t*`
arg resolves back to an index. `trap_SV_RegisterSharedMemory` /
`trap_CG_RegisterSharedMemory` (declared `codemp/game/g_local.h:1976`,
`codemp/cgame/cg_local.h:2421`) register a module-side scratch buffer once at
init (`g_main.c:920`, `cg_main.c:3713`); they send `G_SET_SHARED_BUFFER` /
`CG_SET_SHARED_BUFFER`, whose engine-side handlers store the raw pointer
(`sv.mSharedMemory = VMA(1)`, `sv_game.cpp:940`; `cl.mSharedMemory = VMA(1)`,
`cl_cgame.cpp:1683`) — the same register-once, read-later idiom as
`LocateGameData` — the wrapper `trap_SV_RegisterSharedMemory` is a plain
`syscall(G_SET_SHARED_BUFFER, memory)` (`g_syscalls.c:601-603`). This is
**load-bearing**, not a dead pattern: `G_InitGame` calls
`trap_SV_RegisterSharedMemory(gSharedBuffer)` unconditionally at init
(`g_main.c:920`), the handler stores `VMA(1)` into `sv.mSharedMemory`
(`sv_game.cpp:940`; field `server.h:87`), and that buffer *is* the shared-memory
struct the 12-word-cap escape above reads for high-arity `vmMain` calls
(`C_Trace`/`C_G2Trace` and the `GAME_ICARUS_*` cases) — consumed later by
icarus/RMG. It is therefore a **second registration in the same family as
`LocateGameData`**, with identical per-transport semantics (`NativeDll`/`Static`:
raw pointer cached at registration; `Wasm`: offset re-resolved per access) — see
`SharedGameData` in § Seam definition, SEAM-D4. The per-command reading of the
buffer by specific high-arity `vmMain` handlers is per-subsystem body, out of
scope (→ `docs/subsystems/*`).

## State ownership

Per porting-rules §B, the seam introduces exactly one deliberate global — the
module-side syscall pointer — justified by the §B6 singleton exception because
`vmMain` receives no context argument. Everything else is threaded.

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `syscall` fn-ptr (module) | `codemp/game/g_syscalls.c:8` | `OnceLock<CEngine>` at the module cdylib seam (the §B6 exception) | `dllEntry` export storing `CEngine::new(ptr)` | `&CEngine` passed inward from `vmMain`/`trap_*` |
| `currentVM` | `qcommon/vm.cpp:800` | **eliminated** — each dispatch is explicitly parameterized | — | dispatcher `engine` + `transport` args |
| `lastVM` | `qcommon/vm.cpp:801` | **eliminated** (see SEAM-D1); Raven's `VM_Free` global-clobber bug is **not** reproduced (A4 survey) | — | — |
| `sv.gentities`, `sv.gentitySize`, `sv.num_entities`, `sv.gameClients`, `sv.gameClientSize` | `server/sv_game.cpp:329-334` | `impl SharedGameData` held in the engine's module-host state (§ Seam) | `LocateGameData` handler (`Static`/`NativeDll`: cached base+stride; `Wasm`: re-resolved) | server-state field; owner spine in state-ownership.md (pending) |
| `sv.mSharedMemory` (MP game), `cl.mSharedMemory` (MP cgame) | `server/sv_game.cpp:940`, `client/cl_cgame.cpp:1683`; field `server.h:87` | second `SharedGameData`-family registration in the engine's module-host state (§ Seam), same per-transport contract as `LocateGameData` | `G_SET_SHARED_BUFFER` / `CG_SET_SHARED_BUFFER` handler, registered once (`g_main.c:920`, `cg_main.c:3713`): store `VMA(1)`, return `0` | server/client-state field; owner spine in state-ownership.md (pending) |
| `cgvm.entryPoint` (SP cgame shim) | `code/client/vmachine.cpp:12` | engine-side per-module transport handle (`ModuleTransport::Static`) | module-host on load | dispatcher `engine` arg |
| `uivm` (declared, dead for SP UI) | `code/client/cl_ui.cpp:362` | not modeled — SP UI is a linked call (DEC-07) | — | — |
| SP game `gi` import copy / `globals` export | `code/game/g_main.cpp:879,916` | module-side game state (import copy) + engine-held `game_export_t` handle | `GetGameAPI` | `&game_import_t` inward; `&game_export_t` to engine |
| engine's per-loaded-module transport | (new) | `ModuleTransport { NativeDll \| Static \| Wasm }` in the engine module-host state | module loader (module-loading.md, pending) | passed to each dispatcher call |

## Seam definition

Exact signatures below are FROZEN; porters fill bodies without changing them.
Types marked *forward-declared* are named and shaped elsewhere (state-
ownership.md / module-loading.md, pending) and only referenced here. Subsections
explicitly labeled *(informative)* are the exception: they are **not** frozen
here — their final signatures are frozen in the sibling doc they name, and a
porter treats them as provisional shape, not a contract. (Today only
`SharedGameData` below is informative, frozen in state-ownership.md.)

### Outbound execution trait — `abi-transport`

The typed encode/decode halves already ship and are implemented per call
(`EncodeSysCall` + `DecodeSysCallReturn`, 885×,
`crates/abi-transport/src/generic/transport/syscall.rs:29,38`; `DecodeVmMain` +
`EncodeVmMainReturn`, 34×, `.../transport/vm_main.rs:29,38`); SEAM-D5 keeps these
from `docs/engine-plan.md`. Execution is a new trait over a backend, genericized
over the concrete call type `C`:

```rust
// abi-transport: the single seam every backend implements.
pub trait Execute<C: OutboundSysCall> {
    fn execute(&self, args: C::Args) -> C::Output;
}
```

This replaces the placeholder marker traits `OutboundSysCallExecutor` /
`InboundVmCallExecutor` (`generic/outbound.rs:14`, `generic/inbound.rs:13`),
which have zero impls today; `Execute<C>` is the engine-plan shape kept by
SEAM-D5. `Execute<C>` takes `OutboundSysCallExecutor`'s place in
`crates/abi-transport/src/generic/outbound.rs`; its inbound dual `Dispatch<C>`
(spelled below) replaces `InboundVmCallExecutor` in `generic/inbound.rs`. The
`CEngine` and `Static` backend types are new, both in
`crates/abi-transport/src/generic/engine.rs` (SEAM-D9). Per-call backend
selection differs by capability bound, which is why "how to run a call" cannot
live on `OutboundSysCall` itself.

The inbound dual is a distinct handler trait over `InboundVmCall`, mirroring
`Execute<C>`'s shape, used by the module-side `vmMain` dispatch (SEAM-D8):

```rust
// abi-transport: the inbound seam every module-side handler implements.
pub trait Dispatch<C: InboundVmCall> {
    fn dispatch(&self, args: C::Args) -> C::Output;
}
```

`InboundVmCall` and `OutboundSysCall` expose the same associated surface
(`Args`/`Output`, `generic/inbound.rs`, `generic/outbound.rs`), so `Dispatch<C>`
is the exact mirror of `Execute<C>` with the bound moved to `InboundVmCall`; the
module-side inbound `match` (below) decodes via `DecodeVmMain` and encodes via
`EncodeVmMainReturn` inside each arm.

### `CEngine` — the C-engine outbound backend

```rust
// crates/abi-transport/src/generic/engine.rs (SEAM-D9): wraps the raw syscall ptr.
pub struct CEngine { syscall: RawSyscall }   // RawSyscall = entrypoints.rs:5

impl CEngine {
    pub fn new(syscall: RawSyscall) -> Self { Self { syscall } }

    /// The SOLE unsafe choke point (porting-rules §D11): forwards a runtime
    /// `&[isize]` to the C variadic `syscall` by spelling out a fixed frame.
    unsafe fn raw_syscall_words(&self, import: c_int, words: &[isize]) -> isize;
}

impl<C> Execute<C> for CEngine
where C: EncodeSysCall + DecodeSysCallReturn, C::Import: Into<i32> {
    fn execute(&self, args: C::Args) -> C::Output {
        let t = C::encode_syscall(&args);
        // Module-side encode direction: `From<import enum> for i32` (SEAM-D6).
        let ret = unsafe { self.raw_syscall_words(C::IMPORT.into(), t.args()) };
        C::decode_return(ret)
    }
}
```

`raw_syscall_words` localizes the variadic-ABI wrinkle (a runtime slice cannot be
forwarded to a C variadic fn); it passes a fixed 16-slot frame matching Raven's
outbound trampoline `VM_DllSyscall`'s `int args[16]` (`vm.cpp:363-376`), extras
zero-filled and read only at the indices each trap defines by the engine
dispatcher, exactly as Raven does.

The `execute` body carries `C::IMPORT` (type `C::Import`, e.g. the `#[repr(i32)]`
`MpGameImport`) into `raw_syscall_words`'s `import: c_int` slot through the
`C::Import: Into<i32>` bound and the trivial per-enum `impl From<Enum> for i32`
(SEAM-D6); `c_int` is `i32` on every in-scope target, so `.into()` lands in the
slot directly. This is the module-side (encode) half of the enum↔wire-word
conversion pair; the engine-side (decode) half is `TryFrom<i32>` in the
dispatchers below.

### `Static` — the Rust-engine outbound backend (forward-declared)

The same call defs run against our Rust engine as ordinary safe Rust, `IMPORT`
unused:

```rust
// Shape only; the per-call handler bound and the engine service traits it calls
// are specified in state-ownership.md + docs/subsystems/* (pending).
impl<C> Execute<C> for Static
where C: RunStatic {                       // RunStatic: forward-declared
    fn execute(&self, args: C::Args) -> C::Output { C::run(self, args) }
}
```

### Engine-side runtime transport + dispatchers

```rust
// engine module-host state: chosen per loaded module at runtime (DEC-05).
pub enum ModuleTransport { NativeDll, Static, Wasm }
```

Each module's outbound dispatcher (our `SV_GameSystemCalls` /
`CL_CgameSystemCalls` / `CL_UISystemCalls` equivalents) is a hand-written
**exhaustive `match`** over the existing `#[repr(i32)]` import enum
(`MpGameImport`, `crates/mp/abi/src/game/imports.rs:8`; peers for cgame/ui). The
compiler enforces every variant is handled (SEAM-D3):

```rust
// `engine` = &mut the engine module-host state (forward-declared);
// `args[0]` = syscall number; return is the C `intptr_t` word.
pub fn sv_game_system_calls(engine: &mut ServerGame, args: &[isize]) -> isize;
// each arm: decode words → run against `engine` → encode return, one line;
// unimplemented arms: todo!("Port <trap>")   (porting-rules marker).
```

`args[0]` arrives as a raw wire word, so — as in the inbound dual below — the
exhaustive `match` is preceded by a fallible conversion: the engine-side decode
direction `isize → i32 → MpGameImport::try_from` using the agent-written
exhaustive `TryFrom<i32>` (SEAM-D6). An **unknown trap number** reproduces Raven's dispatcher `default` faithfully
(porting-rules §A2) rather than deciding new behavior: `Com_Error( ERR_DROP,
"Bad game system trap: %i", args[0] )` (`sv_game.cpp:1654`; peers `"Bad cgame
system trap: %i"` `cl_cgame.cpp:1730`, `"Bad UI system trap: %i"`
`cl_ui.cpp:1432`), `Com_Error` = panic per DEC-08. The `match` therefore stays
exhaustive over the valid enum variants while the bad-number fallback lives in
the conversion, not in an arm — symmetric with the inbound `command`-word case.

This `&[isize]` dispatcher receives words only from the transports that emit
them — `NativeDll` (through the C syscall pointer) and `Wasm` (through a wasm
import). A `Static` module is linked into our Rust engine and calls engine
services directly through the `Execute<C> for Static` path (`C::run`, SEAM-D1),
so it never packs syscall words and never enters this dispatcher; `Static`
appears alongside `NativeDll` only in `SharedGameData` (below), where the engine
reads the entity blob outside any trap. Pointer-word *interpretation* inside
decode is therefore parameterized by the emitting transport (SEAM-D4):
`NativeDll` reads a word as a host pointer (`VM_ArgPtr` no-op), `Wasm` routes it
through the host marshaller. The `match` body itself is transport-agnostic and
shared.

The inbound direction (engine→module `vmMain`) is the dual: the `Dispatch<C>`
impl body is a module-side exhaustive `match` over the export enum
(`MpGameExport`, `crates/mp/abi/src/game/exports.rs:9`) decoding via
`DecodeVmMain` and encoding via `EncodeVmMainReturn`. The `command` word arrives
as a raw `c_int` (`AbiCommand`), so the `match` is preceded by a fallible
`c_int → MpGameExport::try_from` conversion (the same `TryFrom<i32>` doctrine,
SEAM-D6); an **unrecognized command** reproduces Raven faithfully (porting-
rules §A2) rather than deciding new behavior — game and UI return `-1`
(`g_main.c:695`, `ui_main.c:624`), cgame raises `CG_Error` then returns `-1`
(`cg_main.c:354-358`, `Com_Error` = panic per DEC-08). The `match` therefore
stays exhaustive over the valid enum variants while the fallback lives in the
conversion, not in an arm. Whether that match is reached through a C `vmMain`
export, a direct linked call, or a wasm export is the module-side compile-time
transport (SEAM-D1).

### Live entrypoint exports (replace the abi-transport stubs)

Current stubs discard/ignore everything (`crates/abi-transport/src/
entrypoints.rs:34` `dllEntry` drops the pointer; `:39-55` `vmMain` returns `0`;
`:59` `GetModuleAPI` and `:72` `GetGameAPI` return `null_mut()`). The live exports
and the per-module `ENGINE` static are declared **per module cdylib shell crate**,
in that crate's `lib.rs` — `crates/jampgame`, `crates/cgame`, `crates/ui`
(SEAM-D9): each shell holds its own `static ENGINE: OnceLock<CEngine>` (SEAM-D1)
and its own module-specific inbound `match` (the `Dispatch<C>` impl body), which
one shared `entrypoints.rs` could not. `abi-transport`'s `entrypoints.rs` keeps
only the raw C-ABI type aliases (`RawSyscall`, `RawVmMain`, …); its stub bodies
are retired by the per-crate live exports below rather than edited in place. Live
shapes (module cdylib `lib.rs`):

```rust
// MP / QVM-shaped modules (module-side, one cdylib per module):
#[no_mangle] pub extern "C" fn dllEntry(syscall: RawSyscall) {
    ENGINE.set(CEngine::new(syscall)).ok();     // the single OnceLock<CEngine>
}
#[no_mangle] pub extern "C" fn vmMain(command: AbiCommand, arg0..arg11: AbiWord)
    -> AbiWord;                                  // → module-side inbound match
#[no_mangle] pub extern "C" fn GetModuleAPI(api_version: AbiCommand,
    import: RawImportTable) -> RawExportTable;   // OpenJK table handshake

// SP game (module-side, table ABI — NO word encoding, SEAM-D2):
#[no_mangle] pub extern "C" fn GetGameAPI(import: *const game_import_t)
    -> *const game_export_t;                     // fills & returns the asserted struct
```

`GetGameAPI` wires the already-laid-out, offset-asserted structs
(`crates/sp/abi/src/game/public/game_import_t.rs`, `game_export_t.rs`) to the
`FunctionTableImport`/`FunctionTableExport` marker traits
(`crates/abi-transport/src/generic/table.rs:5,13`), which are currently opaque
`_private: [u8;0]` placeholders (`crates/sp/abi/src/game/imports.rs`).

### Call-site conventions

Ported logic never spells the transport. MP call sites read `trap::X(...)`, SP
call sites read `gi::X(...)`; both delegate to `ENGINE.execute::<C>(args)` for
the selected backend. Module-side backend selection is a compile-time
`type Engine = CEngine | Static | <wasm>` alias per module crate (SEAM-D1,
engine-plan mechanism kept). SP `gi::X` calls the corresponding
`game_import_t` fn-pointer directly (SEAM-D2) — same `Execute`-style surface to
the caller, faithful wire underneath.

### `SharedGameData` — the register-once/read-later abstraction (informative)

INFORMATIVE here; **frozen in `docs/architecture/state-ownership.md`** (pending).
Shown so the seam's two register-once/read-later idioms — the `LocateGameData`
entity arrays and the `G_SET_SHARED_BUFFER`/`CG_SET_SHARED_BUFFER` command buffer
(§ Raven ground truth) — have a named home; the final method set is settled with
the SV world state there.

```rust
// Engine-internal. Method set mirrors SV_GentityNum / SV_GameClientNum /
// SV_NumForGentity (sv_game.cpp:46-65). Also owns the sv.mSharedMemory /
// cl.mSharedMemory registration (sv_game.cpp:940, cl_cgame.cpp:1683).
pub trait SharedGameData {
    fn gentity(&self, num: usize) -> *mut u8;        // base + stride*num
    fn game_client(&self, num: usize) -> *mut u8;
    fn num_for_gentity(&self, ent: *const u8) -> usize;  // inverse mapping
    fn shared_memory(&self) -> *mut u8;              // registered command buffer
}
```

The raw-pointer-returning accessors sit at the ABI seam and are the documented
porting-rules §D11 seam exemption — engine subsystems wrap them into safe
index/handle access immediately, so no raw pointer aliases in the safe tier
above the seam.

Two implementations, same signature, different contract (SEAM-D4):

- `NativeDll`/`Static`: caches base pointer + stride once at registration and
  does raw arithmetic — faithful to `sv.gentities`/`sv.mSharedMemory`, zero cost.
- `Wasm`: stores the module `Memory` handle + offset/stride and **re-resolves
  `(base+offset, len)` per access with bounds checks**, never caching a base
  pointer, so it survives `memory.grow`.

## Decisions

**SEAM-D1 — Module transport is compile-time per artifact; engine transport is
runtime per-module.** A module cdylib for a real C engine uses `CEngine`
wrapping the raw syscall pointer; the same module crate linked into our Rust
engine uses `Static` (direct engine-service calls); a wasm32 build uses wasm
imports — selected by a `type Engine = ...` alias per module crate (engine-plan
mechanism kept). The engine side is a runtime `ModuleTransport { NativeDll |
Static | Wasm }` per loaded module so one session can mix transports (DEC-05).
The raw syscall pointer lives in exactly one `OnceLock<CEngine>` at the module
cdylib seam — the porting-rules §B6 singleton exception, justified because
`vmMain` takes no context argument — and is threaded inward as `&CEngine`. No
other seam global. *Rejected:* a process-wide `static` engine (violates §B).

**SEAM-D2 — SP keeps its native table wire; no word-encoding layer.** SP game
calls go directly through `game_import_t`/`game_export_t` fn pointers
(`g_public.h:168-529`); MP call sites read `trap::X`, SP read `gi::X`, so ported
logic looks uniform while the wires stay faithful. The
`FunctionTableImport`/`FunctionTableExport` marker traits get wired to the
already-asserted structs in `crates/sp/abi/src/game/public/`. *Rejected:* forcing
tables through the message/word model — it fabricates a wire layer Raven never
had.

**SEAM-D3 — Dispatchers are hand-written exhaustive `match` over the import
enums.** One arm per `MpGameImport`/`CG_*`/`UI*` variant, each a one-line
decode/run/encode; the compiler enforces total coverage, which Raven's C
`switch` never could. Unimplemented arms are `todo!("Port <trap>")` so slice
progress stays greppable (porting-rules marker). *Rejected:* proc-macro /
build-script codegen — it adds a layer to debug for work agents do cheaply.

**SEAM-D4 — WASM host-side marshalling is designed now, implemented
post-parity.** In-module pointers *are* linear-memory offsets, so `Args` structs
and wire shapes are identical across all three transports; only the engine's
*interpretation* of pointer words differs (§ Engine-side dispatchers,
§ `SharedGameData`), so module-side `Args` files are never rewritten and the
wasmtime host lands after native parity (DEC-05.5). *Rejected:* rewriting `Args`
to offset newtypes now — it would ripple through ~1,000 files for a backend not
yet built. (Marshaller scope + wasm32 compile-gate: § Verification strategy.)

**SEAM-D5 — Supersedes `docs/engine-plan.md`.** Its transport half already
shipped and its execution half (`Execute<C>`, `CEngine`, the single unsafe choke
point, `trap::` syntax) is re-specified above with `crates/*` paths, a runtime
engine-side transport enum, WASM as a first-class third transport, and
`RustEngine` reframed as `Static` over engine service traits. *Rejected:* leaving
engine-plan.md authoritative — its `src/` paths and compile-time-only transport
are stale. (Kept/changed specifics are inline in § Seam definition.)

**SEAM-D6 — Enum↔wire-word conversions are hand-written per import/export enum.**
Each import/export enum gets a trivial `impl From<Enum> for i32` (`self as i32`,
the module-side encode direction) plus an agent-written **exhaustive**
`TryFrom<i32>` `match` (the engine-side / module-inbound decode direction,
compiler-checked complete — same doctrine as SEAM-D3). `Execute<C> for CEngine`
therefore carries the bound `C::Import: Into<i32>` (this is the fix for the
reviewed type error), and `raw_syscall_words(&self, import: c_int, words:
&[isize]) -> isize` stays the monomorphic unsafe choke point. Resolves SEAM-Q2,
SEAM-Q3. *Rejected:* `num_enum`/derive — hand-written keeps the wire mapping
greppable and adds no proc-macro to the ABI tier.

**SEAM-D7 — Slice 0 (MP dedicated boot) starts on the `NativeDll` transport.**
`jampgame` boots as a real cdylib with live `dllEntry` + `OnceLock<CEngine>`
wiring through the `native/platform` dylib loader (LOAD-D1, module-loading.md,
pending), proving the drop-in ABI from day one; the same artifact is the OpenJK
live-peer check (DEC-05.2). The `Static` transport bootstrap is explicitly a
**later** slice, after the engine service traits exist (state-ownership.md,
module-loading.md, pending). Resolves SEAM-Q4. *Rejected:* starting on `Static` —
it has no `dllEntry`-analogous bootstrap yet and cannot exercise the drop-in ABI.

**SEAM-D8 — The inbound dual of `Execute<C>` is a distinct trait `Dispatch<C:
InboundVmCall>` mirroring `Execute<C>`'s shape.** One `fn dispatch(&self, args:
C::Args) -> C::Output` (§ Seam definition), replacing the zero-impl
`InboundVmCallExecutor`; each module-side `vmMain` dispatch is a `Dispatch<C>`
impl whose body is the exhaustive export-enum `match`. Resolves SEAM-Q5.
*Rejected:* one unbounded `Execute<C>` serving both directions — it blurs the
outbound/inbound bound and the two dispatch shells.

**SEAM-D9 — Seam type and entrypoint file placement is pinned.** The `CEngine`
and `Static` backend types live in `crates/abi-transport/src/generic/engine.rs`;
each module cdylib shell crate (`crates/jampgame`, `crates/cgame`, `crates/ui`)
holds its own `static ENGINE: OnceLock<CEngine>`, its live `dllEntry`/`vmMain`/
`GetModuleAPI` exports, and its module-specific inbound `Dispatch<C>` `match` in
that crate's `lib.rs`; `abi-transport`'s `entrypoints.rs` keeps only the raw
C-ABI type aliases. Resolves SEAM-Q6. *Rejected:* one shared `entrypoints.rs`
holding all exports — it cannot carry a per-module `OnceLock` or per-module
`match`.

## Verification strategy

Per DEC-09, three oracle-parity layers plus a standing wasm32 compile-gate:

1. **TU-level golden harnesses** (`tools/gp2-oracle` pattern, DEC-09.1):
   round-trip `PASSFLOAT`/word-packing against compiled fragments of the oracle
   `g_syscalls.c` (`PASSFLOAT`, `g_syscalls.c:21-25`) and `FloatAsInt`/`VMF`
   (`sv_game.cpp:384-406`); dispatcher decode goldens comparing our
   `match`-arm decode against `VMA`/`VMF` output for representative traps
   (scalar, string, `trap_Trace` struct-out, float).
2. **Live-peer seam oracle** (DEC-09.2, DEC-05.2): our module cdylib loaded by an
   unmodified OpenJK/retail engine — the end-to-end `dllEntry`/`vmMain`/syscall
   round trip exercised against a real host, the strongest seam check.
3. **Compile-time layers**: the offset/size asserts already green on every
   `#[repr(C)]` seam struct (§D12), plus the exhaustive-`match` dispatchers whose
   totality the compiler proves.
4. **`wasm32` compile-gate (SEAM-D4), standing CI from day one**: module crates
   (`qshared`, `bg`, `uishared`, `game`, `cgame`, `ui`, `abi` tiers) must
   **compile** for `wasm32`; layout asserts are
   `#[cfg(target_pointer_width = "64")]`-gated (e.g.
   `crates/sp/abi/src/game/public/game_import_t.rs`) so the compile-gate stays
   clean under wasm32's 32-bit pointers. Compile-gating is sufficient now because
   module-side semantics are already offset-correct; the host-side marshaller
   that translates the 115 pointer-bearing traps (`docs/abi-traps.md` `ptr` (40) +
   `opaque` (75) buckets) into bounds-checked linear-memory access lands with the
   wasmtime host, after native parity (DEC-05.5, SEAM-D4).

This area is native-track (porting-rules §E): green at every commit, one
function/struct/file per commit, slice-driven.

## Slice hooks

- **Slice 0 (MP dedicated boot).** Runs on the `NativeDll` transport (SEAM-D7):
  `jampgame` cdylib with live `dllEntry` + `OnceLock<CEngine>`, loaded through the
  `native/platform` dylib loader (LOAD-D1, module-loading.md, pending — the one
  external dependency this slice needs frozen elsewhere). Needs frozen here: the
  inbound `Dispatch<C>` `vmMain` dispatch for `GAME_INIT` / `GAME_RUN_FRAME` /
  `GAME_SHUTDOWN` (`g_main.c:517,540,520`); the outbound `G_*` syscall dispatcher
  subset that boot exercises; the native `LocateGameData` path (`SharedGameData`
  native impl). `GAME_INIT` dispatches to `G_InitGame`, which unconditionally
  calls `trap_SV_RegisterSharedMemory(gSharedBuffer)` (`g_main.c:519-520,920`);
  the handler is the second `SharedGameData`-family registration — **store
  `VMA(1)`, return `0`** (`sv_game.cpp:940`, SEAM-D4) — and is part of Slice 0,
  not a later concern. Lifecycle/boot order → lifecycle.md.
- **Later slices.** MP client adds the `CL_CgameSystemCalls` / `CL_UISystemCalls`
  dispatchers and their `vmMain` duals; SP adds the `GetGameAPI` table wiring
  (SEAM-D2) and the cgame vmachine shim (DEC-07); the WASM host marshaller +
  wasmtime backend land after native parity (SEAM-D4, DEC-05.5).

## Open questions

**SEAM-Q1 — How does the registered shared-memory command buffer cross the
seam?** (Escalated in review; the A1 dossier §5.2 footnote mis-classified it as
dead/no-op.) `G_SET_SHARED_BUFFER`/`CG_SET_SHARED_BUFFER` register a module
buffer that the engine stores raw (`sv.mSharedMemory`/`cl.mSharedMemory`,
`sv_game.cpp:940`, `cl_cgame.cpp:1683`) and both sides then use for high-arity
`vmMain` calls (`C_Trace`, `GAME_ICARUS_*`). SEAM-D1..D5 do not cover it: (a) the
owner of `sv.mSharedMemory`/`cl.mSharedMemory` is undecided — the State-ownership
table carries only a placeholder row deferring here; (b) the inbound
`DecodeVmMain` design assumes the 12 `vmMain` words, but these commands read
their args from the buffer instead; (c) no WASM re-resolution story
(`SharedGameData` is scoped to `LocateGameData`'s entity arrays, not this
buffer). Requires a settled decision before FROZEN — escalate to the design
session; do not self-resolve.

**SEAM-Q2 — How is the outbound import number carried from `C::IMPORT` to
`raw_syscall_words`?** The frozen `Execute<C> for CEngine` body calls
`self.raw_syscall_words(C::IMPORT, t.args())`, but `OutboundSysCall::IMPORT` has
type `Self::Import` (e.g. `MpGameImport`, a `#[repr(i32)]` enum,
`crates/mp/abi/src/game/imports.rs:8`) while `raw_syscall_words` takes
`import: c_int` (§ Seam definition). No `Into<c_int>`/conversion bound exists on
`OutboundSysCall` and no import enum carries such an impl today (only
`c_int_to_word(c_int) -> isize`, `crates/abi-transport/src/generic/transport/
syscall.rs:43`, which does not accept an enum). As written the snippet does not
type-check; the fix (an `Import: Into<c_int>` bound plus a per-enum
`impl From<MpGameImport> for c_int`, an `as i32` cast, or another shape) is an
ABI-surface choice. Requires a settled decision before FROZEN — escalate; do not
self-resolve.

**SEAM-Q3 — What implements the fallible wire-word → enum conversions the
dispatchers require?** Both dispatch directions gate their exhaustive `match` on
a fallible conversion — `isize → MpGameImport` engine-side and
`c_int → MpGameExport` module-side (§ Engine-side runtime transport +
dispatchers). `MpGameImport`/`MpGameExport` are plain `#[repr(i32)]` enums with
no conversion impls (`crates/mp/abi/src/game/imports.rs:8`,
`crates/mp/abi/src/game/exports.rs:9`), and no `TryFrom`/derive/crate for this
exists in the tree. The mechanism (hand-written `TryFrom` match arms per enum
vs. an external derive such as `num_enum`) is undecided — it trades
boilerplate-per-enum against a new dependency. Requires a settled decision before
FROZEN — escalate; do not self-resolve.

**SEAM-Q4 — Which transport does Slice 0 start on, and how does `Static`
bootstrap?** Slice 0 lists "`Static` or `NativeDll` transport for `jampgame`"
without choosing (§ Slice hooks), and the two are not interchangeable as a
starting point: `NativeDll` reuses the shown `dllEntry` + `OnceLock<CEngine>`
wiring (§ Live entrypoint exports) but needs `module-loading.md` (pending) to
locate/load the cdylib, while `Static` has **no** described bootstrap analogous
to `dllEntry`/`OnceLock` — how the linked module's `Execute<C> for Static`
backend is constructed and first reached is unspecified. DEC-05 frames
Rust-engine ↔ Rust-module (`Static`) as core scenario 1 and real-engine hosting
(`NativeDll`) as scenarios 2–3. Picking Slice 0's first transport and defining
the `Static` bootstrap depends on `lifecycle.md`/`module-loading.md` (pending)
and requires a settled decision before FROZEN — escalate; do not self-resolve.

**SEAM-Q5 — What is the exact signature of the inbound dual of `Execute<C>`?**
The outbound trait is spelled in full (`Execute<C: OutboundSysCall>`,
§ Seam definition), but its inbound counterpart — which "replaces
`InboundVmCallExecutor` in `generic/inbound.rs`" (SEAM-D5, § Outbound execution
trait) — is only referenced in prose, and doc-standards requires
`## Seam definition` to spell it verbatim. `InboundVmCall` and `OutboundSysCall`
expose the same associated surface (`Args`/`Output`,
`crates/abi-transport/src/generic/inbound.rs`, `.../generic/outbound.rs`), so the
dual is shape-compatible, but its form is a fork the settled decisions do not
pick: one unbounded `Execute<C>` with the trait bound moved to the impl sites and
serving both directions, versus a distinct inbound trait carrying its own name
and `C: InboundVmCall` bound. Requires a settled decision before FROZEN —
escalate; do not self-resolve.

**SEAM-Q6 — Where do the per-module live entrypoint exports, the `ENGINE`
static, and the `CEngine`/`Static` backend types live?** The live `dllEntry` /
`vmMain` / `GetModuleAPI` exports "replace the abi-transport stubs" in the shared
`crates/abi-transport/src/entrypoints.rs` (§ Live entrypoint exports), yet each
module cdylib needs its own `OnceLock<CEngine>` (SEAM-D1) and its own
module-specific inbound `match` (SEAM-D3) — which one shared `entrypoints.rs`
cannot hold. The `CEngine`/`Static` backends are likewise called "new module-side
types in the same crate" (§ Outbound execution trait) without a file. Nothing in
oracle or the settled decisions fixes these Rust file/module placements. Requires
a settled decision before FROZEN — escalate; do not self-resolve.
