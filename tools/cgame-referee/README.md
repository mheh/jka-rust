# cgame trap-shape manifest

Per-trap argument-shape table for the MP cgame syscall surface. It drives the
C6b trap-stream logger shim and the headless replay (DEC-48 ruling 3: full
stream, per-trap serializers, byte-identical bar). One entry per `cgameImport_t`
value: for every argument, what the engine reads or writes through it, plus the
return kind.

`trap-shapes.json` is generated - edit `gen_trap_shapes.py` and re-run:

```
python3 gen_trap_shapes.py > trap-shapes.json
```

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
(oracle). The engine **stores the pointer** and reads through it during **later**
traps and VM calls (the G2 and FX families stage their big argument structs into
this buffer). Size is `MAX_CG_SHARED_BUFFER_SIZE` = 2048 (`cg_public.h:593`); the
Rust cgame types it as `&mut [u8; 2048]` (`crates/mp/cgame/src/trap.rs:3830`).

For replay: the buffer's **contents at each later call** are the payload, not the
one register value handed to this trap. The logger has to snapshot the shared
region at the traps that consume it (or diff it per frame), and the replay must
own a live 2048-byte region and point the engine at it once, rather than
recording a pointer.

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
