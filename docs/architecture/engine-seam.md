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
- `docs/architecture/two-island-model.md` — the A2 state-ownership session
  artifact; STATE-D3 (seam entrypoints/dispatchers are `extern "C-unwind"`) is
  adopted here by SEAM-D12; STATE-D2 (multi-world: engine holds *a* registration,
  not one global) constrains the per-slot trampoline cell (SEAM-D11).
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
`C_G2Trace()` behind `CG_TRACE`/`CG_G2TRACE`, `cg_main.c:243,248`) instead of
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
trampoline; its non-PPC branch `return currentVM->systemCall( &arg );` (`:379`)
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
`ge->apiversion != GAME_API_VERSION` (`server/sv_game.cpp:682-684`).

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
| `currentVM` | `qcommon/vm.cpp:800` | **eliminated as a global** — each dispatch is explicitly parameterized; its one surviving role (bridging the stateless C syscall fn-ptr to per-VM engine state) becomes the per-slot trampoline cell below (SEAM-D11), not a global | — | dispatcher `engine` + `transport` args |
| per-slot `*mut Engine` trampoline cell (engine-side; replaces `currentVM`'s bridging role) | (new; supplants `currentVM` `qcommon/vm.cpp:800`) | `mp_engine_qcommon::ModuleRegistry`'s per-slot `EngineSlot.engine` cell (SEAM-D11; `ModuleRegistry` mirrors oracle `qcommon/vm.cpp`) — one cell **per module slot**, never one global (STATE-D2) | `EngineSlotGuard::enter` at the top of each engine→module call into that slot; cleared on `Drop` | read only by that slot's raw `extern "C-unwind"` syscall trampoline (§ Seam — inbound raw syscall trampoline); the porting-rules §D11 engine-side seam exemption, the twin of the module shell's `OnceLock<CEngine>` |
| `lastVM` | `qcommon/vm.cpp:801` | **eliminated** (see SEAM-D1); Raven's `VM_Free` global-clobber bug is **not** reproduced (A4 survey) | — | — |
| `sv.gentities`, `sv.gentitySize`, `sv.num_entities`, `sv.gameClients`, `sv.gameClientSize` | `server/sv_game.cpp:329-334` | `impl SharedGameData` held in the engine's module-host state (§ Seam) | `LocateGameData` handler (`Static`/`NativeDll`: cached base+stride; `Wasm`: re-resolved) | server-state field; owner spine in state-ownership.md (pending) |
| `sv.mSharedMemory` (MP game), `cl.mSharedMemory` (MP cgame) | `server/sv_game.cpp:940`, `client/cl_cgame.cpp:1683`; field `server.h:87` | second `SharedGameData`-family registration in the engine's module-host state (§ Seam), same per-transport contract as `LocateGameData` | `G_SET_SHARED_BUFFER` / `CG_SET_SHARED_BUFFER` handler, registered once (`g_main.c:920`, `cg_main.c:3713`): store `VMA(1)`, return `0` | server/client-state field; owner spine in state-ownership.md (pending) |
| `cgvm.entryPoint` (SP cgame shim) | `code/client/vmachine.cpp:12` | preserved **only** as the inbound `VM_Call`-shaped dispatch surface into cgame's `vmMain` (pure dispatch, LOAD-D5) held on `ModuleTransport::Static`; **outbound** cgame→engine goes direct via `Execute<C> for Static` (`C::run`), NOT through the shim (SEAM-D1 amendment) | module-host on load | dispatcher `engine` arg (inbound only) |
| `VM_DllSyscall`→`CL_CgameSystemCalls` round-trip (SP cgame outbound shim) | `code/client/vmachine.cpp:36-39` | **not modeled** — internal Raven plumbing with no observable behavior; `Static` subsumes the DEC-07 outbound word path (SEAM-D1 amendment) | — | — |
| `uivm` (declared, dead for SP UI) | `code/client/cl_ui.cpp:362` | not modeled — SP UI is a linked call (DEC-07) | — | — |
| SP game `gi` import copy / `globals` export | `code/game/g_main.cpp:878,916` | module-side game state (import copy) + engine-held `game_export_t` handle | `GetGameAPI` | `&game_import_t` inward; `&game_export_t` to engine |
| engine's per-loaded-module transport | (new) | `ModuleTransport { NativeDll \| Static \| Wasm }`, a field of `mp_engine_qcommon::ModuleRegistry` (the engine module-host subsystem, mirroring oracle `qcommon/vm.cpp`) | module loader (module-loading.md, pending) | passed to each dispatcher call |

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
SEAM-D5. `OutboundSysCallExecutor`'s only in-tree dependent is the message-
syscall helper layer in `generic/message.rs` — `MessageArgs`,
`MessageOutboundSysCall`, and the blanket `impl<T> MessageOutboundSysCallExecutor
for T where T: OutboundSysCallExecutor` (`generic/message.rs:10,32,44`) — which is
part of the same zero-consumer placeholder set: it has **zero**
`MessageOutboundSysCall` call-type impls and **zero** `call_message` callers
(grep-verified; the six `Botlib*ConsoleMessageArgs`/`ChatMessageArgs` syscall
files own their `*Args` structs and encode through `EncodeSysCall` /
`DecodeSysCallReturn` directly, not through this helper). Removing
`OutboundSysCallExecutor` therefore retires `message.rs` in the same step — no
seam consumer depends on it — rather than leaving a dangling blanket impl. `Execute<C>` takes `OutboundSysCallExecutor`'s place in
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
// crate::Type home: `mp_engine_qcommon::ModuleRegistry` (mirrors oracle
// qcommon/vm.cpp's VM subsystem). Chosen per loaded module at runtime (DEC-05).
pub enum ModuleTransport { NativeDll, Static, Wasm }
```

Each module's outbound dispatcher (our `SV_GameSystemCalls` /
`CL_CgameSystemCalls` / `CL_UISystemCalls` equivalents) is a hand-written
**exhaustive `match`** over the existing `#[repr(i32)]` import enum
(`MpGameImport`, `crates/mp/abi/src/game/imports.rs:8`; peers for cgame/ui). The
compiler enforces every variant is handled (SEAM-D3):

```rust
// crate::Type home: `mp_engine_server` (`crates/mp/engine/server`), mirroring
// oracle `server/sv_game.cpp:458` per the one-file-per-Raven-subsystem convention
// (CLAUDE.md) and reinforced by the `&mut ServerGame` server-subsystem receiver —
// distinct from the `mp_engine_qcommon::ModuleRegistry` transport/trampoline
// machinery above, which mirrors a *different* oracle file (`qcommon/vm.cpp`), not
// this dispatcher. The cgame/ui peers (`CL_CgameSystemCalls`/`CL_UISystemCalls`,
// `client/cl_cgame.cpp:644`, `client/cl_ui.cpp:813`) mirror likewise into
// `mp_engine_client` (`crates/mp/engine/client`); both peers land in a later slice
// (§ Slice hooks). `engine` = &mut ServerGame, the server-subsystem module-host
// state (forward-declared); `args[0]` = syscall number; return is the C `intptr_t`
// word.
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
import). The typed `sv_game_system_calls(&mut ServerGame, &[isize])` above is
**not** itself the raw variadic C `syscall` pointer a hosted DLL is handed; the
raw `extern "C-unwind"` adapter that packs varargs into `&[isize]` and reaches the
typed engine state (the inbound dual of `CEngine::raw_syscall_words`) is the
per-slot syscall trampoline frozen below (§ Seam — inbound raw syscall
trampoline, SEAM-D11); it reads its slot's `*mut Engine` cell to reach this typed
dispatcher. A `Static` module is linked into our Rust engine and calls engine
services directly through the `Execute<C> for Static` path (`C::run`, SEAM-D1),
so it never packs syscall words and never enters this dispatcher; `Static`
appears alongside `NativeDll` only in `SharedGameData` (below), where the engine
reads the entity blob outside any trap. Pointer-word *interpretation* inside
decode is therefore parameterized by the emitting transport (SEAM-D4):
`NativeDll` reads a word as a host pointer (`VM_ArgPtr` no-op), `Wasm` routes it
through the host marshaller. The `match` body itself is transport-agnostic and
shared. Module-side `Args` structs and wire shapes are likewise identical across
all three transports — in-module pointers *are* linear-memory offsets — so the
per-call `Args` files are never rewritten per backend; only this engine-side
interpretation and the wasm host marshaller differ (SEAM-D4).

The inbound direction (engine→module `vmMain`) is the dual: the module-side
`vmMain` entrypoint hosts an exhaustive `match` over the export enum
(`MpGameExport`, `crates/mp/abi/src/game/exports.rs:9`); each arm decodes via
`DecodeVmMain`, routes to that command's per-call `Dispatch<C>` impl, and encodes
via `EncodeVmMainReturn` — the exact mirror of the outbound `sv_game_system_calls`
`match` routing to `Execute<C>` handlers. This doc freezes only the
`Dispatch<C>` **trait shape** (SEAM-D8) and the export-enum match routing. The
`Self` **receiver** each `Dispatch<C>` impl reads/writes — the module-side
`GameWorld`-shaped value (informal in `two-island-model.md`) that persists across
`vmMain` calls so `GAME_RUN_FRAME` can mutate state between frames — is **not**
`CEngine` (which carries no game state) and is **not** frozen here: it is the
module-side state-ownership spine, an explicit non-goal owned by
`docs/architecture/state-ownership.md` (§ Scope & non-goals). Where that world
value is constructed and held — and whether it augments the shell's SEAM-D10
inventory or lives elsewhere — is state-ownership.md's to freeze, not this
doc's; SEAM-D10 fixes only the shell's seam surface (`ENGINE`, the exports, the
`Dispatch<C>` match), which is orthogonal to the world the match delegates into.

For the sole purpose of writing this doc's `vmMain` match and `Dispatch<C>`
skeleton, that receiver is **forward-declared** as `GameWorld` — the module-side
owned value already named by `two-island-model.md` (STATE-D1/STATE-D2; e.g.
`g_run_frame(&mut GameWorld, …)`, and STATE-D2 "`GameWorld` is a value") —
exactly as this doc forward-declares `Engine`, `Static`'s `RunStatic`,
`ServerGame`, and `SharedGameData`. Naming it is **not** defining it (no new
decision): the skeleton spells `impl Dispatch<C> for GameWorld` and a `vmMain`
match that holds and dispatches on a `GameWorld`, while that type's fields, its
`GAME_INIT` construction, and its persistent home across bare-`vmMain` calls (the
module-side analog of the shell's `OnceLock<CEngine>` §B6 exception, since
`vmMain` takes no context arg) stay a forward reference frozen in
`state-ownership.md`, not here (§ Scope & non-goals). The `command` word arrives
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

### Inbound raw syscall trampoline (engine-side) — SEAM-D11

When *our* engine hosts a real mod DLL, the module's poisoned `syscall` slot must
be handed a raw C variadic function pointer (Raven's `VM_DllSyscall`,
`qcommon/vm.cpp:363-380`) — the inbound dual of `CEngine::raw_syscall_words`. That
raw fn is stateless, so it needs a channel to the typed per-module engine state.
SEAM-D1 **eliminated** Raven's `currentVM` global (`vm.cpp:800`) that bridged this;
SEAM-D11 replaces it with **one trampoline per module slot**, each reading a
**per-slot** `*mut Engine` cell (never one global — STATE-D2 keeps it
registration-keyed). A `Drop` scope-guard sets that cell for exactly the duration
of each engine→module call into the slot. Frozen shapes, in
`mp_engine_qcommon::ModuleRegistry` (the engine module-host subsystem, mirroring
oracle `qcommon/vm.cpp`):

```rust
// One per hosted module slot in mp_engine_qcommon::ModuleRegistry. The cell holds
// a live `*mut Engine` ONLY while an engine→module call into this slot is on the
// stack. The porting-rules §D11 engine-side seam exemption — the twin of the
// module shell's `OnceLock<CEngine>` (SEAM-D1), one cell per slot (STATE-D2).
struct EngineSlot { engine: Cell<*mut Engine> }            // Engine forward-declared

// RAII: set the slot's cell on entry to an engine→module call, restore on Drop.
struct EngineSlotGuard<'a> { slot: &'a EngineSlot, prev: *mut Engine }
impl EngineSlot {
    fn enter(&self, engine: &mut Engine) -> EngineSlotGuard<'_>;   // cell = engine
}
impl Drop for EngineSlotGuard<'_> { fn drop(&mut self); }          // cell = prev

// The raw fn assigned to the hosted module's `syscall` slot (one monomorphic
// trampoline per slot; e.g. the game slot). Reads `EngineSlot.engine`, reconstructs
// the flat `&[isize]` frame from `&arg` exactly as VM_DllSyscall treats `&arg` as
// an `int[16]` base (vm.cpp:363-376), then calls the typed dispatcher above.
// `extern "C-unwind"` so a Com_Error panic can unwind back through the host's
// live C frames (SEAM-D12).
extern "C-unwind" fn game_syscall_trampoline(arg: isize, ...) -> isize;
//   body (unsafe, the §D11 choke point): let engine = &mut *slot.engine.get();
//                                        sv_game_system_calls(engine, frame_from(&arg))
```

Non-blocking for Slice 0: SEAM-D7 boots our module *inside a real/OpenJK engine*
(which supplies its own syscall pointer), so this engine-side trampoline is
exercised only by the later our-engine-hosting slice — but its shape is frozen
here (this doc's Scope covers engine-side receipt of syscall traffic).

### Live entrypoint exports (replace the abi-transport stubs)

Current stubs discard/ignore everything (`crates/abi-transport/src/
entrypoints.rs:34` `dllEntry` drops the pointer; `:39-55` `vmMain` returns `0`;
`:59` `GetModuleAPI` and `:72` `GetGameAPI` return `null_mut()`). The live exports
and the per-module `ENGINE` static are declared **per module cdylib shell crate**,
in that crate's `lib.rs` (SEAM-D9, SEAM-D10). The shell is the
`crates/jampgame`/`crates/cgame`/`crates/ui` lineage (SP peer: the `jagame`
shell), settled as a **thin cdylib** by SEAM-D10 (resolving SEAM-Q8): the shell
depends on `abi_transport` **and** its logic crate (`mp_game`/`mp_cgame`/`mp_ui`,
SP game tier) and hosts exactly three things — the `ENGINE: OnceLock<CEngine>`
static (SEAM-D1), the live entrypoint exports below, and the module-side
`Dispatch<C>` `match` that delegates into the logic crate. The logic-crate lineage
(`crates/mp/game` etc.) gains **no** entrypoint/`OnceLock` code — it stays
transport-agnostic so the `Static` and `wasm32` builds reuse it; neither lineage
is deleted (SEAM-D10). One shared `entrypoints.rs` could not carry a per-module
`OnceLock` or per-module `match`, so `abi-transport`'s `entrypoints.rs` keeps only
the raw C-ABI type aliases (`RawSyscall`, `RawVmMain`, …); its stub bodies are
retired by the per-crate live exports below rather than edited in place. The
fn-pointer aliases among them — `RawDllEntry`/`RawVmMain`/`RawGetModuleApi`/
`RawGetGameApi` (`crates/abi-transport/src/entrypoints.rs:9-27`) — are the
type spellings of exactly these seam-crossing exports, so they carry
`extern "C-unwind"`, not plain `extern "C"`, under SEAM-D12's rule (no
seam-crossing fn pointer keeps the abort-on-unwind ABI); a later caster/store of a
loaded module's `vmMain` symbol against `RawVmMain` must not hit an ABI-string
mismatch. `RawSyscall`/`RawImportTable`/`RawExportTable` are opaque `c_void`
pointers with no ABI string and are unaffected. The
shipped file is renamed to Raven's filename (`jampgamex86.dll`, …) at package
time (SEAM-D10), not by the crate name. Live shapes (module cdylib `lib.rs`):

```rust
// MP / QVM-shaped modules (module-side, one cdylib per module):
// ABI string is `extern "C-unwind"`, not plain `extern "C"` (SEAM-D12).
#[no_mangle] pub extern "C-unwind" fn dllEntry(syscall: RawSyscall) {
    ENGINE.set(CEngine::new(syscall)).ok();     // the single OnceLock<CEngine>
}
#[no_mangle] pub extern "C-unwind" fn vmMain(command: AbiCommand, arg0..arg11: AbiWord)
    -> AbiWord;                                  // → module-side inbound match
#[no_mangle] pub extern "C-unwind" fn GetModuleAPI(api_version: AbiCommand,
    import: RawImportTable) -> RawExportTable;   // OpenJK-only; contract SEAM-Q7

// SP game (module-side, table ABI — NO word encoding, SEAM-D2):
#[no_mangle] pub extern "C-unwind" fn GetGameAPI(import: *const game_import_t)
    -> *const game_export_t;                     // fills & returns the asserted struct
```

**ABI string (SEAM-D12).** All four exports — and every seam dispatcher/entrypoint
— are `extern "C-unwind"`, not plain `extern "C"`: `Com_Error` becomes a panic
(DEC-08) that must be able to traverse the live C frames of a real host mid-trap,
exactly as Raven throws its C++ string exception through those same frames
(`two-island-model.md`, STATE-D3 session note). The `catch_unwind` boundary sits
at the engine `Com_Frame` (DEC-08, "the frame boundary") — *outside* these module
export functions — so the exports are unwind-transparent and never catch the panic
themselves. This corrects the plain-`extern "C"` spelling this section carried in
draft; STATE-D3 governs. **Follow-up sweep (SEAM-D12).** Pre-existing plain-`extern
"C"` fn-pointer surfaces left by the mechanical type port are swept to
`extern "C-unwind"` in a follow-up code pass tracked as an explicit slice task —
not silently — so no seam-crossing fn pointer keeps the abort-on-unwind ABI. Two
kinds: (a) fn-pointer *fields* (e.g. SP UI `gameinfo_import_t`, renderer
`refexport_t`); and (b) `abi-transport`'s own raw entrypoint fn-pointer *aliases*
`RawDllEntry`/`RawVmMain`/`RawGetModuleApi`/`RawGetGameApi`
(`crates/abi-transport/src/entrypoints.rs:9-27`), currently plain `extern "C"` and
unused, which type exactly the live exports above and so must flip in lockstep
with them (`RawSyscall` and the table-pointer aliases are opaque `c_void` pointers
with no ABI string, unaffected).

**Concrete shell — `crates/jampgame` (SEAM-D10, pins SEAM-Q8).** Its
`Cargo.toml` gains exactly two dependency edges — `abi_transport` **and**
`mp_game` (the logic crate that owns `MpGameImport`/`MpGameExport` and the ported
data model) — and its `crate-type = ["cdylib"]` stays. Its `lib.rs` holds only:

```rust
// crates/jampgame/src/lib.rs — thin cdylib shell (SEAM-D10). Nothing else.
static ENGINE: OnceLock<CEngine> = OnceLock::new();        // SEAM-D1

#[no_mangle] pub extern "C-unwind" fn dllEntry(syscall: RawSyscall) { /* set ENGINE */ }
#[no_mangle] pub extern "C-unwind" fn vmMain(command: AbiCommand, /* arg0..arg11 */)
    -> AbiWord { /* exhaustive export-enum match, each arm routing to a Dispatch<C> impl */ }

// the vmMain match routes each command to its `Dispatch<C>` impl, which calls
// into mp_game (e.g. mp_game::g_init, mp_game::g_run_frame) — the logic crate
// has no entrypoint/OnceLock code of its own (SEAM-D10).
```

`crates/cgame`/`crates/ui` (and the SP `jagame` shell) mirror this against
`mp_cgame`/`mp_ui`/the SP game tier. `crates/mp/game` and its peers stay pure
logic crates — **no** `dllEntry`/`vmMain`/`GetGameAPI`/`OnceLock` — so the
`Static` and `wasm32` backends link them unchanged.

`GetGameAPI` wires the already-laid-out, offset-asserted structs
(`crates/sp/abi/src/game/public/game_import_t.rs`, `game_export_t.rs`) to the
`FunctionTableImport`/`FunctionTableExport` marker traits
(`crates/abi-transport/src/generic/table.rs:5,13`), which are currently opaque
`_private: [u8;0]` placeholders (`crates/sp/abi/src/game/imports.rs`).

`dllEntry` and `vmMain` are the only QVM-module live exports **frozen** here:
Slice 0 (SEAM-D7) exercises exactly those two, and both trace to cited Raven
ground truth (`dllEntry` `g_syscalls.c:14-16`; `vmMain` `g_main.c:515`). The
third slot, `GetModuleAPI`, has **zero occurrences in the Raven oracle** (grep-
verified across `oracle/oracle/**`): Raven 1.01 loads a native module through
`dllEntry`+`vmMain` alone (§ Raven ground truth), so its version-negotiation
check, returned-table content, and call timing cannot be derived from oracle
ground truth. It belongs to the OpenJK-native module-load handshake — the
DEC-05/DEC-05.2 Raven-vs-OpenJK divergence surface — whose contract no cited doc
captures (`tools/closure-prototype/NOTES.md`, the DEC-05 divergence record,
included). Slice 0 does not touch it. Its body therefore stays the current
`entrypoints.rs:59` null stub pending **SEAM-Q7**; the signature above is the
existing `RawGetModuleApi` alias (`entrypoints.rs:25`), not a frozen contract.

### Call-site conventions

Ported logic never spells the transport, and never reaches a seam global
(SEAM-D1 "No other seam global"; porting-rules §B3/§B4): the backend is
**threaded in** as `&Engine`, where `Engine` is the single cfg'd alias owned by
the binding-leaf crate `mp_engine_select` (`crates/mp/engine-select`, SEAM-D13) —
`CEngine` (`NativeDll`) by default, `Static` under Cargo feature `static`, the
wasm backend under `cfg(target_arch = "wasm32")` (that wasm backend's concrete
type/file is **open** — SEAM-Q11). Each logic crate
(`mp_game`/`mp_cgame`/`mp_ui`) depends on `mp_engine_select` and writes
`use mp_engine_select::Engine;` in its `mod trap`, so the logic crate itself
carries **no** cfg and **no** Cargo feature (SEAM-D10 held literally) and `mod
trap` stays in the logic crate with the frozen non-generic wrappers below. SP
needs no select crate: `mod gi` binds `game_import_t` directly (SEAM-D2, always
native) and SP cgame/ui are always `Static`, so their aliases are fixed. The
frozen contract below is the call-site *surface* (`engine.execute(args)`);
`Engine` is bound by `mp_engine_select` per build (SEAM-D13,
`docs/workspace-architecture.md` § Dependency edges).
Each outbound call `C` has one thin module-side wrapper, this exact shape:

```rust
// module crate `mod trap` (SP: `mod gi`), one fn per outbound call C (e.g. GPrint).
// `Engine` = the per-module alias above; `Engine: Execute<C>` holds for every arm.
pub fn X(engine: &Engine, args: <C as OutboundSysCall>::Args)
    -> <C as OutboundSysCall>::Output
{
    engine.execute(args)     // Execute<C>::execute for the selected backend
}
```

MP call sites read `trap::X(engine, args)`, SP call sites read `gi::X(engine,
args)` — `engine` is the `&CEngine` (or `&Static` / `<wasm>`) threaded inward
from the cdylib seam, where `vmMain`/`dllEntry` read the one `OnceLock<CEngine>`
(`ENGINE.get()`) exactly once (State-ownership table: "threaded via `&CEngine`
passed inward"); the call site is **not** a static read. SP `gi::X` resolves `C`
to the corresponding `game_import_t` fn-pointer directly (SEAM-D2) — same
`Execute`-shaped surface to the caller, faithful wire underneath. Any natural-
argument ergonomics above `C::Args` are per-call module-side sugar, out of this
seam's scope (the frozen surface is `engine.execute(args)`).

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
imports — selected by a `type Engine = ...` alias per module crate. (The
concrete per-build selection mechanism and the physical home of that alias and
`mod trap` are settled by **SEAM-D13**: the `mp_engine_select` binding-leaf crate
owns the one cfg'd `type Engine` alias, imported unchanged by each logic crate's
`mod trap`.) The
engine side is a runtime `ModuleTransport { NativeDll |
Static | Wasm }` per loaded module so one session can mix transports (DEC-05).
The raw syscall pointer lives in exactly one `OnceLock<CEngine>` at the module
cdylib seam — the porting-rules §B6 singleton exception, justified because
`vmMain` takes no context argument — and is threaded inward as `&CEngine`. No
other seam global. *Rejected:* a process-wide `static` engine (violates §B).

*Amendment (2026-07-02, SP-cgame contested resolution).* `Static` **subsumes**
the DEC-07 outbound word path for SP cgame: the `vmachine.cpp` shim is preserved
**only** as the inbound `VM_Call`-shaped dispatch surface into cgame's `vmMain`
(pure dispatch, LOAD-D5, module-loading.md, pending); outbound cgame→engine calls
go direct via `Execute<C> for Static` (`C::run`) with **no** word packing.
Raven's `VM_DllSyscall`→`CL_CgameSystemCalls` round-trip (`vmachine.cpp:36-39`) is
internal plumbing with no observable behavior and is not modeled; DEC-07's
"`VM_Call` ABI shape" is read as the **inbound** surface. `ModuleTransport` stays
three-variant (unchanged).

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
post-parity.** Module-side `Args`/wire shapes are transport-identical, so only
the engine's pointer-word interpretation differs (§ Seam definition —
§ Engine-side dispatchers, § `SharedGameData`, which also owns the
`sv.mSharedMemory`/`cl.mSharedMemory` registration); the wasmtime host lands
after native parity (DEC-05.5). Resolves SEAM-Q1. *Rejected:* rewriting `Args` to
offset newtypes now — a ~1,000-file ripple for a backend not yet built.

**SEAM-D5 — This doc supersedes `docs/engine-plan.md`.** engine-plan's transport
half shipped and its execution half is re-specified above (§ Seam definition)
with `crates/*` paths, the runtime engine-side transport enum, first-class WASM,
and `RustEngine` reframed as `Static`. *Rejected:* leaving engine-plan.md
authoritative — its `src/` paths and compile-time-only transport are stale.

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
`InboundVmCallExecutor`; the module-side `vmMain` entrypoint hosts the exhaustive
export-enum `match`, each arm routing to the command's per-call `Dispatch<C>` impl
(the inbound mirror of the outbound `sv_game_system_calls`→`Execute<C>` split).
Resolves SEAM-Q5.
*Rejected:* one unbounded `Execute<C>` serving both directions — it blurs the
outbound/inbound bound and the two dispatch shells.

**SEAM-D9 — Seam type and entrypoint file placement is pinned.** The `CEngine`
and `Static` backend types live in `crates/abi-transport/src/generic/engine.rs`;
each module cdylib shell crate (`crates/jampgame`, `crates/cgame`, `crates/ui`)
holds its own `static ENGINE: OnceLock<CEngine>`, its live `dllEntry`/`vmMain`
exports (the frozen QVM live exports; `GetModuleAPI`'s body is deferred pending
SEAM-Q7), and its module-specific inbound `Dispatch<C>` `match` in that crate's
`lib.rs`; `abi-transport`'s `entrypoints.rs` keeps only the raw C-ABI type
aliases. Resolves SEAM-Q6. *Rejected:* one shared `entrypoints.rs`
holding all exports — it cannot carry a per-module `OnceLock` or per-module
`match`.

**SEAM-D10 — The module shell is a thin cdylib; logic crates stay
transport-agnostic (resolves SEAM-Q8).** `crates/jampgame` (peers `crates/cgame`,
`crates/ui`, and SP's `jagame` shell) are thin cdylib shells: each depends on
`abi_transport` **and** its logic crate (`mp_game`/`mp_cgame`/`mp_ui`, SP game
tier) and hosts exactly three things — the `ENGINE: OnceLock<CEngine>` static
(SEAM-D1), the live entrypoint exports, and the module-side `Dispatch<C>` `match`
delegating into the logic crate. The logic-crate lineage (`crates/mp/game` etc.)
stays transport-agnostic (no entrypoint/`OnceLock` code) so the `Static` and
`wasm32` builds reuse it; neither lineage is deleted. The shipped artifact is
renamed to Raven's filename (`jampgamex86.dll`, …) at package time. This narrows
SEAM-D9 to a concrete crate identity. *Rejected:* deleting one lineage or moving
entrypoint code into the logic crate — it would bind `mp_game` to the `NativeDll`
transport and block `Static`/`wasm32` reuse.

**SEAM-D11 — Our engine reaches a hosted DLL's typed state through one
`extern "C-unwind"` trampoline per module slot, over a per-slot `*mut Engine` cell
set by a `Drop` scope-guard (resolves SEAM-Q9).** Each hosted slot's raw `syscall`
pointer (Raven `VM_DllSyscall`, `vm.cpp:363-380`) is a monomorphic trampoline
that reads its slot's cell to reach the typed dispatcher (§ Seam — inbound raw
syscall trampoline). The guard sets the cell for exactly the duration of each
engine→module call; cells are **per-slot, never one global** (STATE-D2
registration-keying), and the mechanism is the porting-rules §D11 engine-side
seam exemption, twin of the module shell's `OnceLock<CEngine>` (SEAM-D1). This is
the specified replacement for the `currentVM` global SEAM-D1 eliminated.
*Rejected:* one process-wide thread-local/global bridging cell — it breaks the
STATE-D2 multi-world constraint and re-introduces the hidden global §B3 forbids.

**SEAM-D12 — Seam entrypoints/dispatchers are `extern "C-unwind"` (adopts
STATE-D3).** The four live exports (`dllEntry`/`vmMain`/`GetModuleAPI`/
`GetGameAPI`), every seam dispatcher, and the SEAM-D11 trampoline carry the
`-unwind` ABI so a `Com_Error` panic (DEC-08) can traverse a real host's live C
frames mid-trap, as Raven's C++ string exception does today; `catch_unwind` stays
engine-side at `Com_Frame` (DEC-08), outside these exports, so `com_error` panics
legitimately unwind out of module frames. This adopts the already-settled STATE-D3
session note (`two-island-model.md`) — not a new choice; the frozen plain-`extern
"C"` block earlier drafts carried was a defect, and all such frozen seam
signatures are corrected here. **Follow-up sweep:** pre-existing plain-`extern
"C"` fn-pointer surfaces left by the mechanical type port are swept to `-unwind`
in a follow-up code pass tracked as an explicit slice task, not silently — both
fn-pointer *fields* (e.g. SP UI `gameinfo_import_t`, renderer `refexport_t`) and
`abi-transport`'s raw entrypoint fn-pointer *aliases* `RawDllEntry`/`RawVmMain`/
`RawGetModuleApi`/`RawGetGameApi` (`entrypoints.rs:9-27`), which type the live
exports and must not silently diverge from them. *Rejected:* plain
`extern "C"` — unwinding through it is undefined and aborts, breaking DEC-08
recovery when hosting a native DLL.

**SEAM-D13 — The `type Engine` alias lives in a dedicated binding-leaf crate
`mp_engine_select`; logic crates import it and carry no cfg/feature (resolves
SEAM-Q10).** A new crate `crates/mp/engine-select` (package `mp_engine_select`,
depends on `abi-transport` where the concrete `CEngine`/`Static` backends live,
SEAM-D9) holds the single cfg'd `pub type Engine` alias: the wasm arm is selected
by `cfg(target_arch = "wasm32")` (the concrete module-side wasm `Execute<C>`
backend type/file this arm resolves to is **open** — SEAM-Q11), `Static` by a
Cargo feature `static` (enabled
only by static-transport builds), and the default (non-wasm, no feature) is
`CEngine` (`NativeDll`). Each logic crate (`mp_game`/`mp_cgame`/`mp_ui`) depends
on `mp_engine_select` and writes `use mp_engine_select::Engine;` in its `mod
trap`, so the logic crate contains **zero** cfg and **zero** Cargo features
(SEAM-D10 held literally) and `mod trap` stays in the logic crate with the frozen
non-generic wrappers (§ Call-site conventions). Shells select: `jampgame`/`cgame`/
`ui` take the default, a static-linking engine build enables feature `static`.
Because the logic crate must stay transport-agnostic (SEAM-D10) yet its frozen
non-generic wrapper needs a concrete `Engine` in scope, isolating the one
cfg/feature in a binding leaf is the only reconciliation. **Known cost:**
`NativeDll` and `Static` artifacts on the same host triple cannot share one
feature-unified `cargo build --workspace` graph — those builds go per-package
(wasm needs no feature: distinct target triple). SP needs **no** select crate: SP
game's `mod gi` binds the `game_import_t` table directly (SEAM-D2, always native)
and SP cgame/ui are always `Static` — their aliases are fixed. The crate, its
edges, and the selection paragraph are already in `docs/workspace-architecture.md`
(§ Dependency edges). *Rejected:* a Cargo feature on the logic crate itself (it
recompiles per shell and makes the logic crate carry features, breaking SEAM-D10
literally); a generic `Engine` type parameter (contradicts the frozen non-generic
`&Engine` signature); hoisting `mod trap` into the shell crate (a forward
dependency edge `mp_game`→`jampgame`).

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
   **compile** for `wasm32`; because `game`/`cgame`/`ui` resolve `Engine` through
   `mp_engine_select` (SEAM-D13), this presupposes a concrete module-side wasm
   `Execute<C>` backend in scope under `wasm32` from day one, whose type and file
   are **open** — SEAM-Q11. Layout asserts are
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
  the `crates/jampgame` thin cdylib shell (SEAM-D10 — deps `abi_transport` +
  `mp_game`) with live `dllEntry` + `OnceLock<CEngine>` and its `Dispatch<C>`
  match delegating into `mp_game`, loaded through the `native/platform` dylib
  loader (LOAD-D1, module-loading.md, pending — one of two external
  dependencies this slice needs frozen elsewhere; see "Frozen elsewhere" below).
  The crate-topology question that once blocked this
  (SEAM-Q8) is resolved by SEAM-D10: the shell's `lib.rs` receives the live
  exports and match; `crates/mp/game` gains no entrypoint code. **Frozen here**
  (this doc): the inbound `Dispatch<C>` **trait shape** and the export-enum match
  routing for `GAME_INIT` / `GAME_RUN_FRAME` / `GAME_SHUTDOWN`
  (`g_main.c:517,540,520`) (SEAM-D8, § Seam definition); and the **module-side**
  outbound `trap_G_*` wrapper *surface* (`engine.execute(args)`, § Call-site
  conventions), including the
  one-shot registrations `trap_LocateGameData` (`g_syscalls.c:105-108`) and
  `trap_SV_RegisterSharedMemory(gSharedBuffer)` (`g_syscalls.c:601-603`), which
  `GAME_INIT`→`G_InitGame` calls unconditionally at init
  (`g_main.c:519-520,920`). **Frozen elsewhere, blocking this slice** (not this
  doc's to freeze): (a) the `Dispatch<C>` `Self` **receiver** — the owned
  module-side `GameWorld`-shaped value that persists across `vmMain` calls so
  `GAME_RUN_FRAME` can mutate state between frames — and its
  construction/ownership, which is state-ownership.md's spine (§ Scope &
  non-goals; § Seam definition, inbound dual), the same boundary that punts the
  engine-side `SharedGameData` impl below; and (c) the `native/platform` dylib
  loader (LOAD-D1, above). Slice 0 cannot be wired until (a) and (c) are frozen
  in their owning docs/sessions, exactly as it already waits on LOAD-D1. (The
  physical home of `mod trap`/`type Engine` and the mechanism binding the alias to
  `CEngine` for this `NativeDll` build, once tracked here as a blocker, is now
  **frozen in this doc** by SEAM-D13 — the `mp_engine_select` binding leaf,
  default `CEngine` — so it no longer gates the slice.) Because Slice 0 hosts our module **inside a
  real/OpenJK engine** (SEAM-D7), the *engine-side* receivers of those calls —
  `SV_LocateGameData` and the `G_SET_SHARED_BUFFER` handler that stores `VMA(1)`
  and returns `0` (`sv_game.cpp:327-335,940`), i.e. the native `SharedGameData`
  engine-side impl — are the **real host's** C code, not ours; our module only
  emits the outbound calls and trusts the host's existing handler. Building our
  own native `SharedGameData` engine-side impl is therefore **not** in this slice:
  it is the later our-engine-hosting slice's work, the same boundary as the
  SEAM-D11 engine-side trampoline, which is likewise **not** in this slice (a
  real/OpenJK host supplies the syscall pointer). Its shape stays informative
  here, frozen in state-ownership.md (§ `SharedGameData`).
  **Dry-run boundary (the Gate-3 deliverable reachable from this doc + standing
  docs alone).** The skeleton this doc yields for Slice 0 is `crates/jampgame`'s
  `lib.rs` with the live `dllEntry`/`vmMain` exports, the `ENGINE:
  OnceLock<CEngine>` static, and the exhaustive export-enum `vmMain` match routing
  `GAME_INIT`/`GAME_RUN_FRAME`/`GAME_SHUTDOWN` (`g_main.c:517,540,520`) to
  `Dispatch<C> for GameWorld` impls (receiver forward-declared, § Seam
  definition), plus the `trap_G_*` outbound wrapper *surface* (§ Call-site
  conventions). Its terminal **in-scope** plan step is *"`jampgame` compiles and
  its `Dispatch<C>` match + `trap_G_*` wrappers are unit-testable in isolation
  against the forward-declared `GameWorld`."* End-to-end wiring past that step is
  gated on exactly two **named, already-owned** external freezes — not on any hole
  in this doc: (a) `GameWorld`'s definition, `GAME_INIT` construction, and
  persistent home across bare-`vmMain` calls → state-ownership.md; and (c) the
  live host-load of the cdylib and syscall-pointer hand-off that actually invokes
  `dllEntry`/`vmMain` → LOAD-D1, module-loading.md. Both are § Scope & non-goals
  deferrals (not `## Open questions` — each has a named owning doc, per the
  taxonomy in that section), so a Gate-3 dry-run **stops** at the terminal step
  above and cites (a)/(c) as external gates rather than reporting an unanswerable
  question. Lifecycle/boot order → lifecycle.md.
- **Later slices.** MP client adds the `CL_CgameSystemCalls` / `CL_UISystemCalls`
  dispatchers and their `vmMain` duals; our-engine hosting of a real DLL adds the
  SEAM-D11 per-slot trampoline **and** the native `SharedGameData` engine-side impl
  (`SV_LocateGameData` + the `G_SET_SHARED_BUFFER`/`CG_SET_SHARED_BUFFER` handlers,
  `sv_game.cpp:327-335,940`, `cl_cgame.cpp:1683`), both deferred out of Slice 0
  because a real host owns them there (SEAM-D7); SP adds the `GetGameAPI` table
  wiring (SEAM-D2)
  and the SP-cgame `vmMain` inbound dispatch surface — the `vmachine` shim
  preserved **only** for inbound dispatch, outbound going direct via `Static`
  (SEAM-D1 amendment; DEC-07 read as the inbound surface); the WASM host
  marshaller + wasmtime backend land after native parity (SEAM-D4, DEC-05.5).

## Open questions

**Open — returns to a design session:**

- **SEAM-Q7 — `GetModuleAPI` export contract (OpenJK-native load handshake).**
  The QVM-module export list carries `GetModuleAPI(api_version, import) ->
  export_table`, but it has **zero occurrences in the Raven oracle** (grep-
  verified across `oracle/oracle/**`): Raven 1.01 loads native modules through
  `dllEntry`+`vmMain` alone (§ Raven ground truth), and no cited doc
  (`tools/closure-prototype/NOTES.md`, the DEC-05 divergence record, included)
  captures OpenJK's version check, returned-table content, or call timing. Its
  contract is a Raven-vs-OpenJK compatibility decision (DEC-05.2 scope) that
  cannot be settled from oracle ground truth, and it is **not** exercised by
  Slice 0 (SEAM-D7, `dllEntry`/`vmMain` only), so it does not block the first
  slice. Escalated to a design session; the natural home is the OpenJK-native
  load path in `docs/architecture/module-loading.md` (pending). Until settled,
  the export body stays the `entrypoints.rs:59` null stub (§ Live entrypoint
  exports).

- **SEAM-Q11 — Module-side `wasm32` `Execute<C>` backend has no named type or
  file.** SEAM-D13 selects the `type Engine` wasm arm by `cfg(target_arch =
  "wasm32")`, and the verification-strategy `wasm32` compile-gate (§ Verification
  strategy #4, standing CI from day one) requires the logic crates
  (`mp_game`/`mp_cgame`/`mp_ui`, each `use mp_engine_select::Engine;`) to compile
  for `wasm32` — so a concrete module-side backend implementing `Execute<C>` must
  be in scope under `wasm32` now. No decision names it: SEAM-D9 pins only
  `CEngine`/`Static` to `crates/abi-transport/src/generic/engine.rs` (grep confirms
  no `Wasm` backend struct exists in-tree), and SEAM-D4 / DEC-05.5 defer only the
  **engine-side** wasmtime host marshaller to post-parity — a different question
  from what module-side type satisfies `Execute<C>` today — while SEAM-D13 and
  `docs/workspace-architecture.md` § Dependency edges name only "the wasm backend"
  with no type, file, or crate. There is **no oracle ground truth** to derive it
  from: the wasm transport is a jka-rust addition (DEC-05), absent from Raven. The
  choice is genuinely open — the compile-gate could be satisfied by (a) a stub
  backend type in `generic/engine.rs` with a deferred (`todo!()`) `Execute<C>`
  body paralleling `CEngine`/`Static`, (b) a compile-only alias of the wasm arm to
  an existing backend until the wasmtime host lands, or (c) a backend in a
  wasm-specific crate — each a new design decision. Not exercised by Slice 0
  (`NativeDll`, SEAM-D7), so it does not block the first slice. Escalated to a
  design session; the natural home is the wasm-transport design SEAM-D4 / DEC-05.5
  defer to post-parity, but the type/file that satisfies the standing compile-gate
  must be settled before `mp_engine_select`'s wasm arm can be written.

**Resolved (history).** All previously escalated items are resolved; their
resolutions live in the records/sections above:

- **SEAM-Q1** (how the registered shared-memory command buffer crosses the seam)
  — corrected § Raven ground truth (it is load-bearing, not dead: `g_main.c:920`
  → `sv_game.cpp:940`), the `sv.mSharedMemory`/`cl.mSharedMemory` state-ownership
  row, and SEAM-D4 (second `SharedGameData`-family registration).
- **SEAM-Q2, SEAM-Q3** (outbound import-number carry; fallible wire-word→enum
  conversions) — SEAM-D6 (`C::Import: Into<i32>` bound + per-enum `From<Enum> for
  i32` and exhaustive `TryFrom<i32>`).
- **SEAM-Q4** (Slice 0 transport + `Static` bootstrap) — SEAM-D7 (`NativeDll`;
  `Static` bootstrap deferred to a later slice per module-loading.md /
  state-ownership.md, a scoped non-goal here).
- **SEAM-Q5** (inbound dual signature) — SEAM-D8 / `Dispatch<C>` in § Seam
  definition.
- **SEAM-Q6** (entrypoint/`ENGINE`/backend file placement) — SEAM-D9 (the
  per-module-shell *pattern*; the shell's *physical crate identity* is settled by
  SEAM-D10, below).
- **SEAM-Q8** (which physical crate hosts the module shell + its dependency edges)
  — SEAM-D10 (thin `crates/jampgame` shell, deps `abi_transport` + `mp_game`,
  hosting `ENGINE`/exports/`Dispatch<C>`; `crates/mp/game` stays transport-agnostic
  with no entrypoint code; neither lineage deleted; § Live entrypoint exports).
- **SEAM-Q9** (the raw inbound syscall trampoline reaching typed engine state) —
  SEAM-D11 (one `extern "C-unwind"` trampoline per module slot over a per-slot
  `*mut Engine` cell set by a `Drop` scope-guard; § Seam — inbound raw syscall
  trampoline; the specified replacement for the eliminated `currentVM`).
- **SEAM-Q10** (physical home of the `type Engine` alias / `mod trap` + the
  per-build selection mechanism) — SEAM-D13 (the `mp_engine_select` binding-leaf
  crate owns the one cfg'd `type Engine` alias; logic crates import it and carry
  no cfg/feature; § Call-site conventions, § Decisions, and
  `docs/workspace-architecture.md` § Dependency edges).

Remaining deferrals to sibling docs (the `SharedGameData` method set →
state-ownership.md; the module-side `Dispatch<C>` `Self` receiver — the owned
`GameWorld`-shaped value persisted across `vmMain` calls — and its
construction/ownership → state-ownership.md; the `Static` bootstrap slice →
module-loading.md) are scoped non-goals (§ Scope & non-goals), not open
questions. SEAM-Q7 (`GetModuleAPI` OpenJK-load contract) and SEAM-Q11 (the
module-side `wasm32` `Execute<C>` backend type/file) are the genuinely open
items, because no sibling doc yet owns them.
