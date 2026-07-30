# cgame trap-shape manifest

Argument-shape tables for the MP cgame ABI seam, both directions. They drive the
C6b trap-stream RECORDER shim (`shim/`) and the headless replay (DEC-48 rulings
1+3: full bidirectional stream, per-call serializers, byte-identical bar).

- `trap-shapes.json` - the module->engine side (233 `cgameImport_t` traps): for
  every argument, what the engine reads or writes through it, plus the return
  kind.
- `export-shapes.json` - the engine->module side (32 `cgameExport_t` vmMain
  commands): same, plus which arms carry their payload through the engine-
  retained shared buffer, and the pointer-returning arms. See
  'vmMain export shapes' below.
- `shim/` - the standalone recorder cdylib that interposes between openjk.app
  and the real cgame module and journals both streams. See 'The recorder shim'
  and 'Journal format' below.

`trap-shapes.json` is generated - edit `gen_trap_shapes.py` and re-run:

```
python3 gen_trap_shapes.py > trap-shapes.json
```

`export-shapes.json` is hand-authored (32 entries, each cited to
`cg_main.c`/`cl_cgameapi.cpp`); edit the JSON directly.

## Ground truth

The shapes come from the **engine** side, because both the Rust cgame and the
stock cgame run under `openjk.app`. What the engine reads and writes through
each arg is the dispatch:

- `~/Developer/Milo/OpenJK/codemp/client/cl_cgameapi.cpp` -
  `CL_CgameSystemCalls` (line 834+). This is the cite on every entry.

Numbering is `cgameImport_t` in `oracle/codemp/cgame/cg_public.h`. We diffed it
against OpenJK's `cgameImportLegacy_e`: the names and order are byte-identical,
so the trap numbers match across both engines. The 16 shared math/mem traps at
100-115 dispatch under their `TRAP_*` aliases in OpenJK (`sharedTraps_t`,
`qcommon/qcommon.h:302`) but hold the same numbers as `CGAME_MEMSET`..`CGAME_ASIN`.

Cross-checked against `oracle/codemp/client/cl_cgame.cpp` (the retail dispatch);
divergences are catalogued below. `crates/mp/cgame/src/trap.rs` and
`docs/abi-traps.md` were secondary sanity checks on the wrapper signatures.

Note: under `openjk.app` a non-legacy module is actually driven through the
direct `cgi` function-pointer table (`CL_BindCGame`, cl_cgameapi.cpp:1698+),
not the numbered syscall. The direct path passes the **same** arguments - `VMA`
is identity in a native DLL, so `VMA(n) == args[n]` - so the syscall dispatch is
the canonical, complete enumeration of what the engine touches per arg. The
logger keys on the trap number regardless of which path the live module uses.

## Schema

Top-level:

- `schema`, `engine`, `numbering`, `arg_kinds`, `ret_kinds`, `note` - self-describing headers.
- `count` - number of trap entries (233).
- `traps` - the array.

Each trap entry:

```json
{
  "num": 230,
  "name": "CG_GETSNAPSHOT",
  "ret": "scalar",
  "args": [ {"kind": "scalar", "type": "int32"},
            {"kind": "out_buf", "type": "snapshot_t"} ],
  "cite": "OpenJK/codemp/client/cl_cgameapi.cpp:1218",
  "note": "optional, only when something needs saying"
}
```

- `num` - the `cgameImport_t` value.
- `name` - the exact Raven enum identifier.
- `ret` - one of:
  - `void` - dispatch does `return 0;` and the wrapper is `void`.
  - `scalar` - an int/qboolean/count returned in the result word.
  - `handle` - a `qhandle_t`/clip handle/effect id (still an int word, flagged so the replay can pool-remap if needed).
  - `float` - returned via `FloatAsInt(...)`; the result word is float bits, not an int.
- `args` - ordered, one per real argument. **Arg index i in this array is
  `args[i+1]` / `VMA(i+1)` in the dispatch** (`args[0]` is the trap number
  itself). Each arg:
  - `kind` - see below.
  - `type` - the C type the engine casts to (`vec3_t`, `trace_t`, `char*`, ...).
  - `size_of` - engine-native (LP64) `sizeof` of that type in bytes, when fixed.
  - `len_arg` - for variable-length buffers, the 0-based `args[]` index holding
    the byte/element count (so `len_arg: 3` means `args[3]`).
  - `note` - free text.
- `cite` - `file:line` of the dispatch case (OpenJK). Traps with no dispatch
  case cite `cg_public.h`.

### arg `kind` taxonomy

- `scalar` - a value word. `type: "float"` marks a `VMF(n)` word (float bits in
  an int slot) vs `type: "int32"`/`qhandle_t`/etc for a plain int. Ghoul2
  handles are `scalar` with `type: "CGhoul2Info_v*"` - see special cases.
- `in_str` - engine reads a NUL-terminated string at the pointer.
- `in_buf` - engine reads N bytes. Fixed `size_of` (named type), or `len_arg`
  when another arg gives the count.
- `out_buf` - engine writes N bytes back through the pointer. Same size forms.
- `inout_buf` - engine both reads and writes the buffer (e.g. `CG_SNAPVECTOR`
  rounds a vec3 in place; `Cvar_Update` reads name, writes value).
- `double_ptr` - engine reads and/or writes through a pointer-to-pointer. The
  `note` says what happens at each level. This is the ghoul2 slot-address family
  (`CGhoul2Info_v**`) plus the shifted-alloc pair and the precision timer.
- `retained_ptr` - engine **keeps** the pointer past the call and touches it
  later. `CG_SET_SHARED_BUFFER` is the only one. The replay must model it
  specially (keep a live region, re-point the engine), not copy-at-call.

## Special cases

Everything that does not serialize as a plain copy-in / copy-out.

### The engine-retained shared buffer - `CG_SET_SHARED_BUFFER` (344)

`RegisterSharedMemory((char*)VMA(1))` (OpenJK) / `cl.mSharedMemory = (char*)VMA(1)`
(oracle). The engine **stores the pointer** and touches it during **later**
`CGVM_*` vmcalls. The engine writes a parameter struct into the region, then
calls into the module (see the Shared-buffer note in the Journal format
section). No trap dispatch case reads it. Size is `MAX_CG_SHARED_BUFFER_SIZE` =
2048 (`cg_public.h:593`); the Rust cgame types it as `&mut [u8; 2048]`
(`crates/mp/cgame/src/trap.rs:3830`).

For replay: the buffer's **contents at each vmcall** are the payload, not the
one register value handed to this trap. The logger snapshots the shared region
around the vmcall arms that carry a `shared_buffer` field in
`export-shapes.json`. The replay must own a live 2048-byte region and point the
engine at it once, rather than record a pointer.

### Ghoul2 double pointers - the `CGhoul2Info_v**` slot family

These take a pointer to the VM's slot that holds the ghoul2 instance handle. The
engine reads the slot, and for the mutating ones allocates/frees/reallocates the
host-side `CGhoul2Info_v` and writes the new host pointer back into the slot:

- `CG_G2_INITGHOUL2MODEL` (297) - arg0 `CGhoul2Info_v**`: allocates the instance
  vector on first use, writes host ptr back (in/out).
- `CG_G2_CLEANMODELS` (301) - frees the vector, nulls the slot.
- `CG_G2_DUPLICATEGHOUL2INSTANCE` (309) - arg1 `CGhoul2Info_v**`: writes a fresh
  copy's host ptr into the dest slot (arg0 is the source handle).
- `CG_G2_HASGHOUL2MODELONINDEX` (310) - reads the slot, indexes it (read-only).
- `CG_G2_REMOVEGHOUL2MODEL` (311) - removes a model, may reallocate and write back.

For replay: the slot value is a **host pointer that only means anything to the
engine**. The logger records it as an opaque token and the replay maps
recorded-token -> live-handle across the run; it never serializes the pointee.
INIT/DUPLICATE mint a new token (capture the written-back value); CLEAN retires one.

### Ghoul2 instance handles passed by value

Most G2 traps take the instance as `args[1]` cast straight to `CGhoul2Info_v*`
(oracle) or `VMA(1)` handed to a `CL_G2API_*` wrapper as `void*` (OpenJK) - same
64-bit value either way. Recorded as `scalar` / `type: CGhoul2Info_v*`: an opaque
engine-owned token minted by `CG_G2_INITGHOUL2MODEL`, **not** a buffer to copy.
The replay remaps the token to the live handle. `CG_FX_PLAY_BOLTED_EFFECT_ID`
(269) is the odd one - its ghoul2 handle rides `args[3]` raw (the engine does
`*(CGhoul2Info_v*)args[3]`), not `VMA`, but it is the same opaque token.

### Shifted-alloc pair and the precision timer (double_ptr, host-owned block)

- `CG_TRUEMALLOC` (288) - `VM_Shifted_Alloc((void**)VMA(1), size)`: engine
  allocates and writes the host block ptr into the slot. Block is engine-owned.
- `CG_TRUEFREE` (289) - reads the slot ptr, frees it, nulls the slot.
- `CG_PRECISIONTIMER_START` (3) - mints a `timing_c`, writes its host ptr into
  the slot. `CG_PRECISIONTIMER_END` (4) takes that raw host ptr back as `args[1]`
  (not `VMA`) and deletes it. Debug-only profiling; both are effectively
  side-effect-free for parity, but the pointer is retained between the two calls.

### Variable-length / count-by-arg buffers

Recorded with `len_arg`. The awkward one is `CG_CM_MARKFRAGMENTS` (34): arg1 is
an in array of `args[1]` points, arg4 (`pointBuffer`) is an out vertex buffer of
up to `args[4]` verts, arg6 (`fragmentBuffer`) is an out array of up to `args[6]`
`markFragment_t`. `CG_R_ADDPOLYSTOSCENE` (204) reads `args[2] * args[4]`
`polyVert_t` (numVerts per poly x numPolys). `CG_R_GET_BMODEL_VERTS` (221) writes
a vert array whose count the engine returns out-of-band; treat the whole engine
side as opaque-fill and diff the buffer.

### Fixed out-buffers with no length arg (caller-sized)

`CG_G2_GETGLANAME` (306), `CG_G2_GETSURFACENAME` (343) and
`CG_PC_SOURCE_FILE_AND_LINE` (250, filename half) `strcpy`/write into a caller
buffer with no length passed - the caller guarantees `MAX_QPATH`. Serialize as
a NUL-terminated string out, bounded at `MAX_QPATH`.

### No-ops, stubs and dead cases (recorded with arg data anyway)

- `CG_SETCLIENTTURNEXTENT` (237) - `return 0` in both; args declared but unread.
- `CG_R_WEATHER_CONTENTS_OVERRIDE` (348) - assignment commented out in both.
- `CGAME_TESTPRINTINT` (112) / `CGAME_TESTPRINTFLOAT` (113) - `return 0`, args ignored.
- `CG_TESTPRINTINT` (239) / `CG_TESTPRINTFLOAT` (240) - **no dispatch case at
  all**; they fall through to the `default: assert(0)` in both engines. Cited to
  cg_public.h. Unreachable - a live hit means a bug.
- `CG_FX_PLAY_ENTITY_EFFECT` (265) - `assert(0)` in both ("gone!"). Dead; never
  call in replay.
- `CG_G2_GETNUMGOREMARKS` (313), `CG_G2_ADDSKINGORE` (314), `CG_G2_CLEARSKINGORE`
  (315) - guarded by `_G2_GORE`; no-op / return 0 when the engine lacks it.

## OpenJK vs oracle engine divergences

Where the two engines read/write the **same** arg slots the shape is identical;
these are the behavioral splits. Shapes in the JSON follow OpenJK (the live
engine); the note flags the oracle difference.

- **RMG / terrain stubbed in OpenJK.** OpenJK returns 0 and reads nothing for
  `CG_CM_REGISTER_TERRAIN` (345), `CG_RMG_INIT` (346), `CG_RE_INIT_RENDERER_TERRAIN`
  (347); oracle actually runs `CM_RegisterTerrain(...)->GetTerrainId()`,
  `RM_CreateRandomModels(args[1], VMA(2))`, and `RE_InitRendererTerrain(VMA(1))`
  respectively. The arg shapes recorded are oracle's (the richer read); **under
  openjk.app these three consume no arg and return 0**. If the logger runs
  against the oracle engine, RMG_INIT reads `args[1]` + a string at `VMA(2)` and
  the terrain traps read a string at `VMA(1)`.
  (OpenJK cl_cgameapi.cpp:1667-1674; oracle cl_cgame.cpp:1686-1714.)

- **Handler renames, identical shapes** (no replay impact, listed so a diff of
  the two dispatches does not look like a bug):
  - `CG_CM_LOADMAP` (23) - oracle branches on `args[2]` into
    `CM_LoadSubBSP(va("maps/%s.bsp", str+1))`; OpenJK `CL_CM_LoadMap(str, qbool)`.
  - `CG_S_GETVOICEVOLUME` (35) - oracle indexes `s_entityWavVol[args[1]]`; OpenJK
    `CL_S_GetVoiceVolume(args[1])`.
  - `CG_R_SETRANGEFOG` (216), `CG_R_SETREFRACTIONPROP` (217),
    `CG_R_GETDISTANCECULL` (222), `CG_R_GETREALRES` (223) - oracle pokes/reads
    `tr.*` / `glConfig.*` globals inline; OpenJK routes through `re->`.
  - `CG_OPENUIMENU` (238) - oracle `VM_Call(uivm, UI_SET_ACTIVE_MENU, args[1])`;
    OpenJK `CL_OpenUIMenu(args[1])`.
  - `CG_MILLISECONDS` (2) - oracle `Sys_Milliseconds`; OpenJK `CL_Milliseconds`.
  - `CG_CVAR_SET` (7), `CG_REMOVECOMMAND` (20), `CG_SENDCLIENTCOMMAND` (21),
    `CG_S_SHUTUP` (45) - oracle inlines the state write; OpenJK calls a
    `CGVM_*`/`CL_*` shim. Same args.
  - The whole G2 family - oracle casts `args[n]` straight to `CGhoul2Info_v*` and
    calls `G2API_*`; OpenJK passes `VMA(n)` to `CL_G2API_*` wrappers. Same handle
    value, same buffers. (See ghoul2 special cases above.)

- **`CG_SETUSERCMDVALUE` (235)** takes 8 value args in both. Oracle stashes
  `args[8]` into `cl_bUseFighterPitch` then calls `CL_SetUserCmdValue` with the
  first 7; OpenJK passes all 8 to `_CL_SetUserCmdValue`. Shape identical (8 scalars).

## Things flagged as not certain

- **`CG_R_GET_BMODEL_VERTS` (221) out-buffer length** is engine-internal (the
  vert count is not an arg; the engine fills up to its own cap). Recorded as an
  opaque `vec3_t` out-array with no `len_arg`; the logger should diff the region
  rather than copy a known length. (cl_cgameapi.cpp:1326.)

- **`CG_R_INPVS` (262) arg2 mask** is `(byte*)VMA(3)` - the engine reads a PVS/area
  mask but the length is not passed; retail callers hand it the snapshot
  `areamask` (`MAX_MAP_AREA_BYTES`). Recorded as `in_buf byte` with no size;
  confirm the caller's buffer size before fixing a copy length.
  (cl_cgameapi.cpp:1355.)

- **`CG_CM_MARKFRAGMENTS` (34) out buffers** (`pointBuffer`, `fragmentBuffer`) are
  sized by `args[4]`/`args[6]` as maxima, not exact counts - the engine writes
  however many it produced and returns the fragment count. Copy at the max, or
  diff. (cl_cgameapi.cpp:1024.)

- **`CG_G2_COLLISIONDETECT` (299) rayStart/rayEnd** (args 7, 8) are passed in and
  the API may write adjusted values back; recorded as `out_buf` to be safe (the
  cache variant reads them as `in`). Confirm against `G2API_CollisionDetect` if a
  parity miss shows up here. (cl_cgameapi.cpp:1506.)

## vmMain export shapes (`export-shapes.json`)

The trap manifest above covers imports only. `export-shapes.json` is the
engine->module half: one entry per `cgameExport_t` value (0..31,
`cg_public.h:352-440`), classified the same way. Ground truth is
`oracle/codemp/cgame/cg_main.c:190-359` (the `vmMain` switch) and the OpenJK
`CGVM_*` wrappers that call it (`cl_cgameapi.cpp`), cited on every entry.

Schema differs from the trap side in three ways:

- **Arg indexing is direct.** Export arg index N is the raw vmMain word `argN` -
  there is no trap-number prefix, so no `+1` (`args[0]` on the trap side).
- **The shared buffer is the real payload for most arms.** Only 11 of the 32
  arms carry data in their arg words. The rest stage their struct through
  `cg.sharedBuffer` - the engine writes a `TCG*` struct into the 2048-byte region
  before the VM call and reads results back after (`C_PointContents`,
  `C_GetLerpData`, `C_Trace`, ... `cg_main.c:362-580`). Those entries carry a
  `shared_buffer` field (`in` / `out` / `inout`); the recorder dumps the whole
  2048-byte region at ENTER and/or EXIT rather than trying to read the arg words.
- **`ret` adds two pointer kinds.** `ptr_opaque` and `ptr_deref` below.

### The pointer-returning arms

Four arms return a pointer the engine casts back from the `int`/`intptr_t`
result word (the four `return (int)ptr` casts at `cg_main.c:232,237,294,297`,
widened to `intptr_t` in the oracle build):

- **`CG_GET_GHOUL2` (12)** - `return (int)cg_entities[arg0].ghoul2`, a
  `CGhoul2Info_v*` host handle. **Dead in OpenJK**: grep of `codemp/` finds no
  `CGVM_` wrapper reading it, and Raven's own comment (`cg_main.c:232-234`) says
  the effect bolting it feeds "is actually not used at all". Recorded
  `ptr_opaque`: the return word is logged as a token, no deref.
- **`CG_GET_MODEL_LIST` (13)** - `return (int)cgs.gameModels`, the module's model-
  handle array. Also dead in OpenJK (no `CGVM_` caller). `ptr_opaque`.
- **`CG_GET_ORIGIN_TRAJECTORY` (23)** - `return (int)&cg_entities[arg0].nextState.pos`,
  a `trajectory_t*`. **Live**: `CGVM_GetOriginTrajectory` (`cl_cgameapi.cpp:236`)
  hands it to the ROFF system, which reads AND writes through it -
  `SetLerp(originTrajectory, TR_LINEAR/TR_STATIONARY, ...)` at
  `RoffSystem.cpp:903,924,939` (call sites `:876,:1023`). Recorded `ptr_deref`:
  the recorder dereferences 36 bytes (`trajectory_t`, `q_shared.h:2654-2660`) at
  EXIT.
- **`CG_GET_ANGLE_TRAJECTORY` (24)** - `return (int)&cg_entities[arg0].nextState.apos`,
  same `trajectory_t*` read/written by `CGVM_GetAngleTrajectory`
  (`cl_cgameapi.cpp:245`) via the same ROFF `SetLerp`. `ptr_deref`, 36 bytes at
  EXIT.

The two plain out-vec arms are ordinary `out_buf`: `CG_GET_ORIGIN` (21) and
`CG_GET_ANGLES` (22) `VectorCopy` into the engine's `arg1` vec3
(`cl_cgameapi.cpp:216,226`). `CG_ROFF_NOTETRACK_CALLBACK` (25) reads an `in_str`
at `arg1` (`cl_cgameapi.cpp:254`).

## The recorder shim (`shim/`)

A standalone cargo project (its own empty `[workspace]` table keeps it out of the
main jka-rust workspace). openjk.app dlopens the cdylib as the cgame module
(`libcgamearm64.dylib`; stage it as `cgamearm64.dylib`). On `dllEntry` it stores
the engine syscall, dlopens the REAL module named by `JKA_SHIM_REAL_CGAME`, and
hands it OUR logging trampoline in place of the engine syscall. Every `vmMain`
and every trap is journaled, then forwarded unchanged.

- **Env.** `JKA_SHIM_REAL_CGAME` (required, the real module to forward to; loud
  stderr + a `MARKER` record + abort if unset/unloadable). `JKA_SHIM_JOURNAL`
  (journal path; defaults to `cgame-shim-journal.bin` **beside the real module**,
  never a silent `/tmp` default). Journal is buffered, flushed on CG_SHUTDOWN.
- **The variadic seam is in C.** `src/trampoline.c` holds the variadic
  trampoline the real module calls and the forward to the real engine - stable
  Rust can neither define nor correctly call a C-variadic fn on Apple arm64
  (stack-passed va-args), the same reason
  `crates/mp/engine/qcommon/src/vm/game_syscall_trampoline.c` exists. It grabs 16
  words the way oracle `VM_DllSyscall` does (`vm.cpp:363-377`) and calls the Rust
  loggers around the forward.
- **Serializers are table-driven, generated at build time.** `build.rs` parses
  `../trap-shapes.json` and `../export-shapes.json` and emits the Rust shape
  tables into `$OUT_DIR/manifest_tables.rs` - chosen over a checked-in generated
  `.rs` so the tables can never drift from the manifests (edit the JSON, rebuild,
  the shim serializes the new shape). `serde_json` is a build-dependency only, so
  the runtime cdylib ships no JSON parser.
- **Reentrancy.** Traps re-enter vmMain (CG_INIT -> trap_UpdateScreen ->
  CG_DRAW_ACTIVE_FRAME). The journal lock is only ever held to write one record,
  never across a forwarded call, so the nested chain cannot deadlock; the bracket
  records nest naturally (LIFO). ENTER/EXIT are paired by a per-thread seq stack.
- **Never aborts a session on bad data.** An unclassifiable command still gets
  its ENTER/EXIT bracket plus a `MALFORMED` record with the raw words; the diff
  tooling surfaces it later. Every foreign-memory read null-checks and caps its
  length.

Build + prove the interpose chain without the engine:

```sh
cd tools/cgame-referee/shim
cargo test        # tests/interpose.rs drives the oracle dylib through the shim
```

The test points `JKA_SHIM_REAL_CGAME` at
`tools/cgame-oracle/build/liboraclecgame.dylib` (run
`tools/cgame-oracle/build.sh` first if absent), installs a stub engine syscall,
drives `vmMain` with an unknown command, and asserts (a) the `CG_ERROR` syscall
reached the stub THROUGH the shim's trampoline and (b) the journal holds the
`VMCALL_ENTER / SYSCALL_ENTER / SYSCALL_EXIT / VMCALL_EXIT` bracket.

## Journal format

The contract the headless replay harness is written against. Length-prefixed
little-endian binary, all multi-byte integers LE.

**File header:** magic `CGSHIMJ1` (8 bytes) + `u32` format version (currently 1).

**Then a sequence of records**, each:

```
u32  payload_len          bytes that follow this field (skip-ahead length)
u8   rec_type             1..6 (below)
u64  seq                  monotonic; ENTER and its EXIT share one seq
...  body                 rec_type-specific
```

`rec_type`:

- `1` VMCALL_ENTER, `2` VMCALL_EXIT - the engine->module direction (`vmMain`).
- `3` SYSCALL_ENTER, `4` SYSCALL_EXIT - the module->engine direction (traps).
- `5` MALFORMED - an unclassifiable command; raw words only, forwarding continued.
- `6` MARKER - a free-text note (module loaded, fatal setup failure).

Nesting is real and encoded by sequential bracketing: a trap fired from inside a
vmMain call writes its SYSCALL_ENTER/EXIT between that call's VMCALL_ENTER and
VMCALL_EXIT; a trap that re-enters vmMain nests another VMCALL pair inside its
own. Records are strictly ordered on the single engine thread.

### Bodies

A **raw word block** is `u8 count` then `count` * `i64` words.

A **blob section** is `u16 blob_count` then that many blobs, each:

```
u8   arg_index            manifest arg position; 0xFF = shared buffer, 0xFE = return deref
u8   blob_kind            1 in_str, 2 in_buf, 3 out_buf, 4 inout_buf,
                          5 double_ptr_slot, 6 shared_buffer, 7 ret_deref
u32  blob_len
...  blob_len bytes
```

**VMCALL_ENTER (1):** `i64 cmd` (cgameExport_t), raw word block (12 words:
arg0..arg11), blob section. Blobs: `in_str`/`in_buf` args per the export shape,
plus a `shared_buffer` (0xFF) dump of the 2048-byte region when the arm's
`shared_buffer` is `in`/`inout`.

**VMCALL_EXIT (2):** `i64 cmd`, `i64 ret`, blob section. Blobs: `out_buf`/
`inout_buf` args (engine-written vec3 for GET_ORIGIN/GET_ANGLES), a `shared_buffer`
dump when `shared_buffer` is `out`/`inout`, and a `ret_deref` (0xFE) blob of the
pointed-to bytes for the `ptr_deref` arms (36-byte `trajectory_t` for
GET_ORIGIN_TRAJECTORY / GET_ANGLE_TRAJECTORY). `ptr_opaque` returns carry no
blob - the token is the bare `ret` word.

**SYSCALL_ENTER (3):** `i64 cmd` (cgameImport_t trap number), raw word block (16
words: `args[0]`=trap number, `args[1..15]` grabbed by the trampoline), blob
section. Blobs per `trap-shapes.json`: `in_str`, `in_buf` (fixed `size_of`, or
`len_arg`-counted with `size_of` as the element stride - counted wins when both
are present - or trap 204's `args[2]*args[4]` product special-case),
`inout_buf`, and `double_ptr_slot` (0..; the slot value BEFORE the engine writes
a new host token). A trap carries a `shared_buffer` (0xFF) dump only when its
manifest entry sets `shared_buffer: true`. Today no trap sets it.
`CG_SET_SHARED_BUFFER` (344) registers the region pointer for the export-side
dumps.

**SYSCALL_EXIT (4):** `i64 cmd`, `i64 ret`, blob section. Blobs: `out_buf`/
`inout_buf` args (engine-written) and `double_ptr_slot` (the engine-written token
after INIT/DUPLICATE/etc.).

**MALFORMED (5):** `i64 cmd`, raw word block. Emitted alongside (not instead of)
the ENTER/EXIT bracket when the command has no manifest shape.

**MARKER (6):** `u32 text_len`, `text_len` UTF-8 bytes.

### Shared-buffer note

The shared traffic is on the export (vmcall) side only. No trap dispatch case
reads the shared region. Each engine touch of `cl.mSharedMemory` writes a
parameter struct and then calls INTO the module through a `CGVM_*` vmcall:
`SFxHelper::CameraShake`/`GetOriginAxisFromBolt` (FxSystem.cpp:100-118),
the FX trace and G2 mark helpers (FxSystem.h:97-151), the scheduler vector
data (FxScheduler.cpp:144,914,1083), the console-command block
(cl_keys.cpp:689), and the automap input (cl_input.cpp:525).

- **Exports** dump per the `shared_buffer` field in `export-shapes.json`. This
  set is authoritative.
- **Traps** dump only when a manifest entry sets `shared_buffer: true`. Today no
  trap sets it. The first recorder build used a `CG_G2_*`/`CG_FX_*` name-family
  heuristic here. That build wrote 4KB around every bolt-matrix call and
  journaled 3GB in 12 seconds. The ground truth above replaced it.

## The headless replay referee

The reader for this journal is `crates/cgame/tests/replay_referee.rs` (DEC-48
rulings 1,2,5). It drives ONE cgame module dylib from a recorded trace and
byte-diffs the module's outgoing trap stream against the recording. The
recording came from the oracle cgame under the live engine, so the recorded
module-side stream IS the oracle reference stream: the oracle dylib replayed
against its own recording must be byte-identical (the self-check), and the Rust
cgame dylib replayed against the same recording gives the verdict.

Two `--ignored` tests, run serially (the game slot + module statics are process
singletons):

```
cargo test -p cgame --release -- --ignored --test-threads=1
```

The trace path comes from `JKA_TRACE` (default
`$HOME/Developer/jka/trace-swoop1.bin`); both tests SKIP with a clear message
when no trace is present (DEC-48.4 - traces stay out of git). The shape tables
are parsed at runtime from the two manifests through the shared
`tools/cgame-referee/shapes.rs` (included via `#[path]`, so crates/cgame takes no
dependency on the shim crate).

Scalar arg words are compared at 32-bit width and pointer-token scalars (ghoul2
handles, type ending in `*`) are not compared at all: the cgame VM args are
32-bit ints, the variadic trampoline grabs 64-bit words, and the high 32 bits are
stack garbage that differs run-to-run. Pointer-arg words are ASLR-dependent and
never compared - the serialized pointee blob is the deterministic witness.

### The `len_arg` 32-bit count bug (fixed 2026-07-30)

`buf_len` / `special_count` read the count word as a full 64-bit `isize`. Count
args are 32-bit ints, so the high 32 bits are variadic-trampoline garbage. For
`CG_FS_GETFILELIST` (17) the `bufsize` word recorded as `0x500_0000_0800` (low 32
= 2048) - past `MAX_BLOB`, so the recorder journaled an EMPTY out-buffer for the
vehicle list (seq 4) and the saber list (seq ~105). Both `serialize.rs` (the
recorder) and `shapes.rs` (the replay) now mask the count to `i32`, matching the
engine dispatch (it casts `args[n]` to `int`). Traces recorded before this fix
carry empty `len_arg` buffers and cannot self-check past the first
`CG_FS_GETFILELIST`; re-record to validate.

### The ROFF SetLerp gap

`CG_GET_ORIGIN_TRAJECTORY` (23) and `CG_GET_ANGLE_TRAJECTORY` (24) return a
`trajectory_t*` that the ROFF system reads AND writes through AFTER the vmcall
returns - `SetLerp(originTrajectory, TR_LINEAR/TR_STATIONARY, ...)` at
`RoffSystem.cpp:903,924,939` (call sites `:876,:1023`). The shim records a
`ptr_deref` dump of the 36-byte `trajectory_t` at vmcall EXIT, but it cannot see
the engine's later SetLerp write, so the replay does not reproduce it. Replay
diffs the deref at EXIT and will show a finding here for any fixture that runs
ROFFs. Modeled gap, not solved - it only matters for ROFF fixtures.

### Replay diff policy (what is NOT compared)

Three exclusions from the byte-identical bar, each because the bytes are host
state, not module behavior:

- **Pointer arg words** are never compared. The pointee blob is the witness.
  Scalar words compare on the low 32 bits only (the trampoline's high 32 are
  stack garbage).
- **`double_ptr` slot contents** are host tokens. The replay serves recorded
  tokens back, so a compare is a tautology. A mis-shaped word here derefs a
  token and faults - this is how the ANGLEOVERRIDE/CLEANMODELS manifest
  numbering swap was caught (fixed; the full 233-name sweep against
  `cg_public.h` now matches exactly).
- **`CG_FX_ADDPRIMITIVE` (279)** compares masked ranges only: each vert's
  origin plus the mShader/mSetFlags/mKillTime tail. The rest of the 348-byte
  `effectTrailArgStruct_t` (q_shared.h:2615-2620) is uninitialized caller
  stack - Raven fills fields per `mSetFlags` and the engine reads them the
  same way, so unset bytes differ run to run by design.

The harness reads and writes module memory through mach `vm_read_overwrite` /
`vm_write`, so a quirk address returns empty instead of SIGBUS. Set
`JKA_REPLAY_TRACE=1` for a per-trap call trace while debugging.

Bar proven 2026-07-30: the oracle dylib replayed against its own swoop1
recording - 33.5M records, 16.77M syscalls, 2,148 vmcalls - with ZERO findings.
