# A4 — Module Loading Ground-Truth Dossier

Scratch dossier (gitignored) for the module-loading/linking design session feeding
`decisions.md` DEC-05 (native dylib + static + WASM transports; retail-DLL hosting
i686-only) and DEC-07 (SP static via vmachine shim). Every claim below cites
`oracle/oracle/<path>:<line>`; current-repo claims cite the actual crate/doc path.

---

## 1. MP loading chain (`codemp/`)

### VM_Create — `oracle/oracle/codemp/qcommon/vm.cpp:471-597`

`vm_t *VM_Create( const char *module, int (*systemCalls)(int *), vmInterpret_t interpret )`
(`vm.cpp:471-472`).

- **Bad-parms fatal**: `!module || !module[0] || !systemCalls` → `Com_Error(ERR_FATAL, ...)` (`vm.cpp:480-482`).
- **Slot reuse by name** (`vm.cpp:485-489`): linear scan of `vmTable[MAX_VM]` via
  `Q_stricmp(vmTable[i].name, module)`; if a live match exists it's returned
  as-is (no reload).
- **Free-slot allocation** (`vm.cpp:492-503`): first slot with `name[0]==0`;
  fatal `"VM_Create: no free vm_t"` if all `MAX_VM` slots are full
  (`vm.cpp:499-500`). `vm->name` and `vm->systemCall = systemCalls` set
  (`vm.cpp:505-506`).
- **fs_restrict demo override** (`vm.cpp:508-513`): if `interpret==VMI_NATIVE`
  and cvar `fs_restrict` is truthy, forces `VMI_COMPILED` — demo builds can
  never load native DLLs.
- **Native attempt** (`vm.cpp:515-525`): `Sys_LoadDll(module, &vm->entryPoint,
  VM_DllSyscall)`; success returns the vm immediately. Failure prints
  `"Failed to load dll, looking for qvm.\n"` and falls through to QVM —
  a non-fatal fallback, not an error.
- **QVM path** (`vm.cpp:527-587`): `filename = "vm/%s.qvm"` (`vm.cpp:528`),
  `FS_ReadFile` (`vm.cpp:530`; `NULL` header → `VM_Free`+`return NULL`);
  header byte-swap + magic/length validation (`vm.cpp:538-550`, fatal on bad
  header); data segment rounded to next power of 2 so all data ops can be
  mask-protected (`vm.cpp:552-557`); `vm->dataBase=VM_Alloc(dataLength)`,
  `vm->dataMask=dataLength-1` (`vm.cpp:560-561`); `VM_Compile` vs
  `VM_PrepareInterpreter` chosen by `interpret>=VMI_COMPILED`
  (`vm.cpp:578-584`); `VM_LoadSymbols` (`vm.cpp:590`); stack placed at top of
  segment: `programStack=dataMask+1; stackBottom=programStack-STACK_SIZE`
  (`vm.cpp:593-594`, `STACK_SIZE=0x20000` at `vm.cpp:469`).
- Native vs bytecode dispatch downstream is simply "is `vm->entryPoint`
  non-NULL" (set only by the native path, `vm.cpp:518`).

### Sys_LoadDll (Win32) — `oracle/oracle/codemp/win32/win_main.cpp:811-887`

`void * QDECL Sys_LoadDll( const char *name, int (QDECL **entryPoint)(int, ...), int (QDECL *systemcalls)(int, ...) )`
(`win_main.cpp:811-812`).

- **Filename construction** (`win_main.cpp:826`): `Com_sprintf(filename,
  sizeof(filename), "%sx86.dll", name)` — i.e. `"jampgame"→"jampgamex86.dll"`,
  `"cgame"→"cgamex86.dll"`, `"ui"→"uix86.dll"`. **No `ARCH_STRING` macro
  exists anywhere in the oracle tree** (confirmed by grep — zero hits); the
  suffix is a hardcoded literal per-platform-suffix (`x86`), not a
  cross-platform macro.
- `Sys_UnpackDLL(filename)` runs first (`win_main.cpp:849-852`) — extracts the
  DLL from a pk3 to disk if a pure server requires it; failure aborts with
  `NULL`.
- **Search order** (`win_main.cpp:855-873`), each step tried in sequence,
  first hit wins:
  1. Bare `LoadLibrary(filename)` — CWD / default DLL search
     (`win_main.cpp:855`).
  2. `fs_basepath` + `fs_game` via `FS_BuildOSPath` (`win_main.cpp:858-863`).
  3. `fs_cdpath` + `fs_game`, only if `fs_cdpath` is non-empty
     (`win_main.cpp:866-869`).
  4. `return NULL` (`win_main.cpp:871-873`) — **no `fs_homepath` fallback in
     this function.** `VM_Create` treats `NULL` as "fall back to QVM"
     (`vm.cpp:519-524`), not fatal.
  `FS_BuildOSPath` (`oracle/oracle/codemp/qcommon/files.cpp:479-498`) builds
  `"<base>/<game>/<qpath>"` via a 4-slot ring buffer of static path strings.

### GetProcAddress handshake — `win32/win_main.cpp:879-887`

```
879  dllEntry = ( void (QDECL *)( int (QDECL *)( int, ... ) ) )GetProcAddress( libHandle, "dllEntry" );
880  *entryPoint = (int (QDECL *)(int,...))GetProcAddress( libHandle, "vmMain" );
881  if ( !*entryPoint || !dllEntry ) {
882      FreeLibrary( libHandle );
883      return NULL;
884  }
885  dllEntry( systemcalls );
887  return libHandle;
```

Two exported symbols, `"dllEntry"` and `"vmMain"`, both required or the module
is immediately unloaded. `dllEntry`'s signature (`win_main.cpp:815`): `void
(QDECL *)( int (QDECL *syscallptr)(int, ...) )` — a one-shot setter that hands
the module the engine's `VM_DllSyscall` trampoline (the same function pointer
passed into `Sys_LoadDll` from `VM_Create`, `vm.cpp:518`) so the module's own
trap stubs (`cg_syscalls.cpp`-style) can call back into the engine. Unix
mirrors this with `dlsym(...,"dllEntry")` / `dlsym(...,"vmMain")`
(`codemp/unix/unix_main.c:421,428`), handshake call at `unix_main.c:444`.

### VM_Call (arg marshaling) — `vm.cpp:787-829`

`int QDECL VM_Call( vm_t *vm, int callnum, ... )` (`vm.cpp:787`); swaps
`currentVM`/`lastVM` around the call for re-entrancy (`vm.cpp:799-801,
826-827`).

- **Native path** (`vm.cpp:807-819`): varargs pulled via a `va_arg` loop into
  a fixed `args[16]`, then re-expanded as a genuine C call
  `vm->entryPoint(callnum, args[0..15])` (`vm.cpp:816-819`), `QDECL`/`__cdecl`
  convention; return value is `entryPoint`'s direct return.
- **Interpreted/compiled path** (`vm.cpp:820-823`): `&callnum` is passed
  directly as `int *args` to `VM_CallCompiled`/`VM_CallInterpreted`, relying
  on the native stack layout placing the variadic args contiguously after
  `callnum` (documented in the function header comment, `vm.cpp:766-777`) —
  read off the stack rather than copied into a buffer.
- **Reverse direction** — `VM_DllSyscall` (`vm.cpp:363, 378-380`): a module
  calling back into the engine takes `&arg` (address of its first vararg) and
  forwards it as `int *parms` to `currentVM->systemCall(&arg)`
  (`vm.cpp:379`). A `__linux__ && __powerpc__` branch (`vm.cpp:364-377`)
  instead copies varargs via `va_arg` since that ABI doesn't guarantee
  contiguous stack layout — an existing precedent for "the naive
  address-of-first-vararg trick is platform-fragile," relevant to any Rust
  reimplementation.

### VM handle bookkeeping

- `vmTable[MAX_VM]` — `vm.cpp:29`; `MAX_VM` — `vm.cpp:28`, `#define MAX_VM 3`
  (exactly one slot per game/cgame/ui).
- `currentVM` — `vm.cpp:24`, `lastVM` — `vm.cpp:25` (both `vm_t*`, file-scope
  globals).
- `vm_t` struct — `oracle/oracle/codemp/qcommon/vm_local.h:111-146`; forward
  decl in `qcommon/qcommon.h:273`.
- Alternate statically-linked backend `qcommon/vm_console.cpp:4-9` keeps its
  own `vmTable`/`currentVM`/`lastVM` with **fixed** slot indices
  `UI_VM_INDEX=0, CG_VM_INDEX=1, G_VM_INDEX=2` (`vm_console.cpp:66-68`) — used
  on platforms that bake all three modules into one binary (console/Mac SP
  lineage; see §4).

### Unload/restart flow

- **VM_Free** — `vm.cpp:605-626`: if `vm->dllHandle`, calls
  `Sys_UnloadDll(vm->dllHandle)` then zeroes the struct (`vm.cpp:607-610`);
  unconditionally clears the **global** `currentVM=NULL; lastVM=NULL`
  (`vm.cpp:624-625`) regardless of which `vm` was freed — a shared-state trap
  worth flagging for the Rust redesign (freeing any one VM clobbers the
  "current VM" bookkeeping for all of them).
- **VM_Restart** — `vm.cpp:391-458`, verbatim comment `vm.cpp:398`: *"DLL's
  can't be restarted in place."* Native path (`vm.cpp:399-409`): saves
  `systemCall`/`name`, `VM_Free(vm)`, then `vm = VM_Create(name, systemCall,
  VMI_NATIVE)` — a full destroy+recreate cycle disguised as a "restart." QVM
  path (`vm.cpp:412-451`): re-reads/re-validates `vm/<name>.qvm` and
  re-copies the initial data segment into the **existing** `dataBase` without
  reallocating (per header comment `vm.cpp:386-389`, purpose-built so
  `map_restart` skips the realloc) — genuinely different semantics from
  native.
- **Callers, jampgame** (`oracle/oracle/codemp/server/sv_game.cpp`):
  - `SV_InitGameProgs` (`sv_game.cpp:1731-1753`), doc comment
    `sv_game.cpp:1725-1730`: *"Called on a normal map change, not on a
    map_restart"* — `gvm = VM_Create("jampgame", SV_GameSystemCalls,
    (vmInterpret_t)(int)Cvar_VariableValue("vm_game"))` (`sv_game.cpp:1750`).
  - `SV_ShutdownGameProgs` (`sv_game.cpp:1666-1673`), doc comment
    `sv_game.cpp:1659-1664`: *"Called every time a map changes"* —
    `VM_Call(gvm, GAME_SHUTDOWN, qfalse); VM_Free(gvm); gvm=NULL;`.
  - `SV_RestartGameProgs` (`sv_game.cpp:1708-1721`), doc comment
    `sv_game.cpp:1701-1706`: *"Called on a map_restart, but not on a normal
    map change"* — `VM_Call(gvm, GAME_SHUTDOWN, qtrue); gvm =
    VM_Restart(gvm);` (`sv_game.cpp:1712-1715`) — **no** `VM_Free`/`VM_Create`
    pair here.
  - Real callers in `oracle/oracle/codemp/server/sv_init.cpp`: `SV_SpawnServer`
    (comment `sv_init.cpp:461-465`: *"NOT called for map_restart"*) calls
    `SV_ShutdownGameProgs()` at `sv_init.cpp:484` (before `CM_LoadMap`) then
    `SV_InitGameProgs()` at `sv_init.cpp:662` (after map load) — i.e.
    **destroy+recreate on every normal map change**. `SV_Shutdown` (comment
    `sv_init.cpp:921-928`) calls `SV_ShutdownGameProgs()` at
    `sv_init.cpp:946` with no re-create (engine/server exit). `map_restart`
    console command calls `SV_RestartGameProgs()` at
    `oracle/oracle/codemp/server/sv_ccmds.cpp:296` — the in-place path.
  - **Conclusion: jampgame is fully destroyed+recreated on every normal map
    change, in-place-restarted only on `map_restart`, and torn down for good
    at server shutdown.**

- **Callers, cgame** (`oracle/oracle/codemp/client/cl_cgame.cpp`,
  `client/cl_main.cpp`):
  - `CL_InitCGame` (`cl_cgame.cpp:1743-1774`) picks `interpret` from
    `cl_connectedToPureServer`/`vm_cgame` (`cl_cgame.cpp:1760-1770`); `cgvm =
    VM_Create("cgame", CL_CgameSystemCalls, interpret)`
    (`cl_cgame.cpp:1771`).
  - `CL_ShutdownCGame` (`cl_cgame.cpp:595-604`): `VM_Call(cgvm, CG_SHUTDOWN);
    VM_Free(cgvm); cgvm=NULL;` (`cl_cgame.cpp:601-603`).
  - `CL_ShutdownAll` (`cl_main.cpp:657-682`) calls `CL_ShutdownCGame()`
    (`cl_main.cpp:667`) and `CL_ShutdownUI()` (`cl_main.cpp:669`) together;
    called from `CL_FlushMemory` (`cl_main.cpp:737`) and top-level
    `CL_Shutdown` (`cl_main.cpp:2741`).
  - `CL_FlushMemory` (`cl_main.cpp:734-767`, doc comment `cl_main.cpp:725-731`:
    *"Called by CL_MapLoading, CL_Connect_f, CL_PlayDemo_f, and
    CL_ParseGamestate — the only ways a client gets into a game"*) calls
    `CL_ShutdownAll()` then, after clearing the hunk, `CL_StartHunkUsers()`
    (`cl_main.cpp:766`), which reinits renderer/sound/UI.
  - `CL_DownloadsComplete` (`cl_main.cpp:1460-1501`) calls `CL_FlushMemory()`
    (`cl_main.cpp:1497`) then `CL_InitCGame()` directly
    (`cl_main.cpp:1501`) — this runs at the end of **every** gamestate
    download, i.e. on every connect and every map-to-map transition while
    connected. (Two earlier `CL_FlushMemory` call sites in `CL_PlayDemo_f`
    `cl_main.cpp:569` and `CL_Connect_f` `cl_main.cpp:1177` are commented out
    with *"now called in CL_DownloadsComplete."*)
  - `CL_Vid_Restart_f` (`cl_main.cpp:1311-1366`) calls `CL_ShutdownUI()`
    (`cl_main.cpp:1322`) and `CL_ShutdownCGame()` (`cl_main.cpp:1324`)
    directly, tears down/reinits the renderer, `CL_StartHunkUsers()`
    (`cl_main.cpp:1357`, re-inits UI), and if still connected,
    `CL_InitCGame()` again (`cl_main.cpp:1362`).
  - A bare `CL_Disconnect()` (`cl_main.cpp:837-901`) does **not** tear down
    cgame/ui itself.
  - **Conclusion: cgame is destroyed+recreated on every map load (initial
    connect and map-to-map transitions via `CL_DownloadsComplete →
    CL_FlushMemory → CL_InitCGame`), on `vid_restart`, and torn down for good
    at engine shutdown. Not freed by plain disconnect.**

- **Callers, ui** (`oracle/oracle/codemp/client/cl_ui.cpp`,
  `client/cl_main.cpp`):
  - `CL_InitUI` (`cl_ui.cpp:1462-1489`): `uivm = VM_Create("ui",
    CL_UISystemCalls, interpret)` (`cl_ui.cpp:1478`), then version-checks via
    `VM_Call(uivm, UI_GETAPIVERSION)` against `UI_API_VERSION`
    (`cl_ui.cpp:1484-1487`).
  - `CL_ShutdownUI` (`cl_ui.cpp:1444-1453`): `VM_Call(uivm, UI_SHUTDOWN);
    VM_Call(uivm, UI_MENU_RESET); VM_Free(uivm); uivm=NULL;`.
  - `CL_StartHunkUsers` (`cl_main.cpp:2445-2473`) calls `CL_InitUI()`
    (`cl_main.cpp:2471`) only if `!cls.uiStarted` (`cl_main.cpp:2469`); called
    from `CL_FlushMemory` (`cl_main.cpp:766`), `CL_Vid_Restart_f`
    (`cl_main.cpp:1357`), and initial engine startup.
  - `CL_ShutdownAll` calls `CL_ShutdownUI()` (`cl_main.cpp:669`) and resets
    `cls.uiStarted=qfalse` (`cl_main.cpp:677`), which is what makes
    `CL_StartHunkUsers` reload it.
  - **Conclusion: ui is NOT session-persistent.** Because `CL_FlushMemory`
    unconditionally runs `CL_ShutdownAll()` (frees `uivm`, clears
    `uiStarted`) then `CL_StartHunkUsers()` (recreates it), and
    `CL_FlushMemory` itself fires on every map load via
    `CL_DownloadsComplete`, **ui is destroyed+recreated on every map load, in
    lockstep with cgame** — plus again on vid_restart, torn down for good at
    engine shutdown.

### Symbol/ABI / calling-convention details

- `QDECL` macro: empty by default (`oracle/oracle/codemp/game/q_shared.h:139`);
  Windows override `#define QDECL __cdecl` (`q_shared.h:152`).
- `vmMain` prototype shape — identical "command + 12 int args" in all three
  modules: `game/g_main.c:515`, `cgame/cg_main.c:190`, `ui/ui_main.c:579` all
  declare `int vmMain( int command, int arg0, ..., int arg11 )`. No dedicated
  prototype lives in `g_public.h`/`cg_public.h`/`ui_public.h` (checked, zero
  hits) since normally it's resolved dynamically; static-link builds forward
  `extern int vmMain(...)` at `qcommon/vm_console.cpp:72,78,84` and
  `win32/win_main_console.cpp:593`.
- Engine-side call target: `vm_local.h:123` — `int (QDECL *entryPoint)( int
  callNum, ... );` (variadic). `VM_Call` invokes it with 16 trailing args
  (`vm.cpp:816-819`) against a callee fixed-arity at 13 ints total
  (`command` + 12); `QDECL`/`__cdecl`'s caller-cleans-stack convention makes
  the excess args harmless on 32-bit x86 — **this convention detail does not
  generalize to non-cdecl/non-x86 targets** (relevant to any non-x86 native
  host and to WASM, where there is no stack-cleanup slop to exploit).
- API version constants: `GAME_API_VERSION=8` (`game/g_public.h:11`);
  `UI_API_VERSION=7` (`ui/ui_public.h:6`, checked via `UI_GETAPIVERSION`
  `ui_public.h:217` and `cl_ui.cpp:1484-1487`); `CGAME_IMPORT_API_VERSION=5`
  (`cgame/cg_public.h:54` — no matching runtime export-version check found
  for cgame, unlike ui).
- 32-bit pointer assumption: `typedef int vmptr_t;` (`vm_local.h:99`);
  `dataBase`/`dataMask` are plain 32-bit fields (`vm_local.h:135-136`).

### VM_ArgPtr / VM_ExplicitArgPtr — the pointer-translation precedent

`VM_ArgPtr` — `vm.cpp:640-654`:

```
640  void *VM_ArgPtr( int intValue ) {
641      if ( !intValue ) { return NULL; }
645      if ( currentVM==NULL ) return NULL;
648      if ( currentVM->entryPoint ) {
649          return (void *)(currentVM->dataBase + intValue);
650      } else {
652          return (void *)(currentVM->dataBase + (intValue & currentVM->dataMask));
653      }
654  }
```

`VM_ExplicitArgPtr` (`vm.cpp:742-758`) is the same logic parameterized on an
explicit `vm_t *vm` — but its null guard still checks the **global**
`currentVM` (`vm.cpp:748`) rather than the passed-in `vm` (a quirk inherited
from id's original code), while the actual pointer math at `vm.cpp:752-757`
correctly uses `vm->entryPoint`/`vm->dataBase`/`vm->dataMask`.

- `intValue==0` → host `NULL` (`vm.cpp:641-643, 743-745`): offset 0 of a VM's
  data segment is unaddressable as a real pointer.
- **Native DLL VMs** (`entryPoint` truthy): `dataBase + intValue`, **unmasked**
  (`vm.cpp:649, 753`) — native code is trusted, not memory-isolated.
- **QVM (bytecode) VMs**: `dataBase + (intValue & dataMask)` (`vm.cpp:652,
  756`) — the `& dataMask` bitmask (`dataMask = dataLength-1`, set at
  `vm.cpp:561` because the data segment is rounded to a power of 2 at
  creation) is the actual memory-safety fence confining any VM-forged offset
  to its own segment. Reused identically by `BotVMShift` (`vm.cpp:657-677`)
  and `VM_Shifted_Alloc`/`VM_Shifted_Free` (`vm.cpp:679-740`) against the
  global `gvm`.
- **This is the direct historical precedent for a wasm32 module boundary**:
  QVM already implements "guest gives a 32-bit linear offset; host masks it
  into its own owned buffer and adds a base" — exactly the shape a
  `wasm32` linear-memory translation needs (module offset + host-owned
  `Vec<u8>`/`Memory` + bounds check in place of the power-of-2 mask, since
  wasm linear memory isn't necessarily power-of-2 sized).

---

## 2. Module creation/destruction sites (already folded into §1's citations)

Summary table (see §1 for full call-chain citations):

| Module | Create | Destroy | Cadence |
|---|---|---|---|
| jampgame | `sv_game.cpp:1750` (`SV_InitGameProgs`) | `sv_game.cpp:1671-1672` (`SV_ShutdownGameProgs`) | destroy+recreate every normal map change (`sv_init.cpp:484,662`); in-place `VM_Restart` only on `map_restart` (`sv_game.cpp:1712-1715`, `sv_ccmds.cpp:296`); final teardown at `SV_Shutdown` (`sv_init.cpp:946`) |
| cgame | `cl_cgame.cpp:1771` (`CL_InitCGame`) | `cl_cgame.cpp:601-603` (`CL_ShutdownCGame`) | destroy+recreate every map load via `CL_DownloadsComplete → CL_FlushMemory → CL_ShutdownAll/CL_InitCGame` (`cl_main.cpp:1497,1501`); also on `vid_restart` (`cl_main.cpp:1322,1324,1362`); not freed by plain disconnect |
| ui | `cl_ui.cpp:1478` (`CL_InitUI`) | `cl_ui.cpp:1450-1453` (`CL_ShutdownUI`) | same cadence as cgame — `CL_FlushMemory`'s `CL_ShutdownAll`+`CL_StartHunkUsers` pair recreates it every map load (`cl_main.cpp:667,669,2471`), plus vid_restart (`cl_main.cpp:1322,1357`) |

Original task citations (`sv_game.cpp:1750`, `cl_cgame.cpp:1771`,
`cl_ui.cpp:1478`) all verified exact.

---

## 3. Unix/Mac Sys_LoadDll variants

### Unix — `oracle/oracle/codemp/unix/unix_main.c:323-447`

Filename construction (`unix_main.c:341-356`):

```c
getcwd(curpath, sizeof(curpath));
#if defined __i386__
#ifndef NDEBUG
  snprintf (fname, sizeof(fname), "%si386-debug.so", name); // 344
#else
  snprintf (fname, sizeof(fname), "%si386.so", name);        // 346
#endif
#elif defined __powerpc__
  snprintf (fname, sizeof(fname), "%sppc.so", name);          // 349
#elif defined __axp__
  snprintf (fname, sizeof(fname), "%saxp.so", name);          // 351
#elif defined __mips__
  snprintf (fname, sizeof(fname), "%smips.so", name);         // 353
#endif
```

**No `.mp.` infix and no `ARCH_STRING` macro** — confirmed absent everywhere
under `codemp/unix` and `codemp/qcommon`. Naming is
`<module-name><arch-suffix>.so`: `"jampgame"→jampgamei386.so`,
`"cgame"→cgamei386.so`, `"ui"→uii386.so` (release x86 Linux; `-debug` variant
in debug builds). Search order: `fs_basepath`+`fs_game` first
(`unix_main.c:379-384`), `fs_cdpath`+`fs_game` fallback
(`unix_main.c:395-396`); opened via `dlopen(fn, RTLD_NOW)`
(`unix_main.c:384`); symbols via `dlsym(...,"dllEntry")`
(`unix_main.c:421`)/`dlsym(...,"vmMain")` (`unix_main.c:428`), handshake call
`unix_main.c:444`.

For contrast, Windows uses the identical `<name><arch>` scheme with `.dll`
(`win_main.cpp:826`, `"%sx86.dll"`) — no MP-specific infix on either
platform.

### Mac

- **No `codemp/mac/` directory exists at all.** The only Mac-named file under
  `codemp/` is `oracle/oracle/codemp/null/mac_net.c`, which is a pure
  networking stub (`NET_StringToAdr`, `Sys_SendPacket`) — no
  `Sys_LoadDll`/`dlopen`/`CFBundle`/`.dylib` code anywhere. **MP (jamp) has no
  Mac dynamic-loading backend in the oracle source.**
- SP's `oracle/oracle/code/mac/mac_main.c:65-73` does have a `Sys_LoadDll`,
  but it's a **hard-linked no-op stub**, not a real loader:
  ```c
  void *Sys_LoadDll( const char *name, int (**entryPoint)(int, ...),
      int (*systemCalls)(int, ...) ) {
      dllEntry( systemCalls );
      *entryPoint = vmMain;
      return (void *)1;
  }
  ```
  Companion comments at `mac_main.c:39,47`: *"we are hard-linked in, so no
  need to load anything"* (for game and UI respectively). No `.dylib` string
  or `CFBundle`/`NSModule` call appears anywhere in the file.
- **No cross-platform `sys_loadlib` unifier exists** anywhere in the oracle
  tree (grepped for `*loadlib*`/`*sysload*`/`codemp/sys*`/`code/sys*` — zero
  hits). Each platform back-end (`win32/`, `unix/`, `null/` for MP;
  `win32/`, `mac/` for SP) defines its own freestanding `Sys_LoadDll`.

| Platform | File | Suffix scheme |
|---|---|---|
| Win32 (MP) | `codemp/win32/win_main.cpp:826` | `"%sx86.dll"` |
| Unix (MP) | `codemp/unix/unix_main.c:344-353` | `"%si386[-debug].so"` / `ppc` / `axp` / `mips` variants |
| Mac (SP only; no MP equivalent) | `code/mac/mac_main.c:65-73` | none — hard-linked stub, nothing loaded |

The `.mp.<arch>.so`/`.dylib` naming hypothesized in the task brief does not
appear anywhere in oracle; real scheme is bare `<module-name><arch-string>.<ext>`
concatenation with no MP infix.

---

## 4. SP (`code/`) module loading

### Sys_GetGameAPI — `oracle/oracle/code/win32/win_main.cpp:478-547`

```
478  Sys_GetGameAPI
483  void *Sys_GetGameAPI (void *parms)
484  {
485      void  *(*GetGameAPI) (void *);
...
489      const char *gamename = "jagamex86.dll";   // _M_IX86 build
```

Alpha build uses `"jagameaxp.dll"` (`win_main.cpp:500`). **Search order is
much simpler than MP's** — no `fs_game`/`fs_basepath`/`fs_cdpath` cvars
referenced anywhere in this function (confirmed by grep, zero hits):

1. `cwd/<debugdir>/jagamex86.dll` (`win_main.cpp:515`; `debugdir` is
   `release`/`shdebug`/`debug` per build config, `win_main.cpp:491-497`).
2. If that fails, `cwd/jagamex86.dll` (`win_main.cpp:524`).
3. Both fail → `Com_Error(ERR_FATAL, "Couldn't load game")`
   (`win_main.cpp:536`).

Symbol lookup: `GetGameAPI = (void *(*)(void *))GetProcAddress(game_library,
"GetGameAPI")` (`win_main.cpp:540`); on success `return GetGameAPI(parms)`
(`win_main.cpp:546`).

### sv_game.cpp / sv_init.cpp wiring

- Forward decl: `extern void *Sys_GetGameAPI( void *parms);` —
  `oracle/oracle/code/server/sv_game.cpp:29`.
- `SV_InitGameProgs` (`sv_game.cpp:473-497`, body starts `sv_game.cpp:478`)
  builds a large `game_import_t import` struct (hundreds of function-pointer
  fields — G2API/renderer/RMG hooks, e.g. `sv_game.cpp:590-662`), then:
  ```
  669   ge = (game_export_t *)Sys_GetGameAPI (&import);
  671   if (!ge)
  672       Com_Error (ERR_DROP, "failed to load game DLL");
  675   //hook up the client while we're here
  678   if (!VM_Create("cl"))
  679       Com_Error (ERR_DROP, "failed to attach to the client DLL");
  ```
  (`sv_game.cpp:669-680`). `ge` (`game_export_t*`) is version-checked
  (`sv_game.cpp:682-684`) and used for `ge->Init(...)`
  (`sv_game.cpp:690`) and the rest of the server's lifetime
  (`ge->RunFrame`, `ge->ConsoleCommand`, `ge->gentities`/`gentitySize` — see
  `SV_GentityNum`, `sv_game.cpp:51-58`).
- Caller: `SV_InitGameProgs();` — `oracle/oracle/code/server/sv_init.cpp:406`,
  inside `SV_SpawnServer`, right after `sv.state = SS_LOADING`
  (`sv_init.cpp:403`).

**Key fact for DEC-07**: immediately after loading `jagamex86.dll`,
`SV_InitGameProgs` also fakes a `VM_Create("cl")` call
(`sv_game.cpp:676,678`) to wire up cgame — **not** a separate DLL load. This
`VM_Create` is the SP-only shim defined in
`oracle/oracle/code/client/vmachine.h:72-91` (see below), unrelated to MP's
real `VM_Create` in `qcommon/vm.cpp`.

### Retail cgame path — `code/win32/win_main.cpp:552-570`

```
552  Sys_LoadCgame
557  void * Sys_LoadCgame( int (**entryPoint)(int, ...), int (*systemcalls)(int, ...) )
558  {
561      dllEntry = ( void (*)( int (*)( int, ... ) ) )GetProcAddress( game_library, "dllEntry" );
562      *entryPoint = (int (*)(int,...))GetProcAddress( game_library, "vmMain" );
563      if ( !*entryPoint || !dllEntry ) {
564          FreeLibrary( game_library );
565          return NULL;
566      }
568      dllEntry( systemcalls );
569      return game_library;
570  }
```

Critically this calls `GetProcAddress` on the **same** `HINSTANCE
game_library` already used for `jagamex86.dll` (declared `win_main.cpp:459`)
— **not a distinct cgame DLL**. Reachable only via the fake `VM_Create("cl")`
in `code/client/vmachine.h:72-91` (see next section), called exactly once
from `sv_game.cpp:676,678`.

Confirmed by project files: `code/game/game.vcproj` is `ConfigurationType="2"`
(DLL), `OutputFile` = `jagamex86.dll`, and compiles **both** `g_main.cpp`
(game logic) **and** `../cgame/cg_main.cpp` + `../cgame/cg_syscalls.cpp`
(cgame logic, including `cg_dllEntry`/`vmMain`) into that one DLL.
`code/starwars.vcproj` (`ConfigurationType="1"`, exe, output `jasp.exe`)
compiles the UI implementation directly (`ui_atoms.cpp`, `ui_shared.cpp`,
`ui_main.cpp`, `ui_syscalls.cpp`) into the exe — no separate `cgamex86.dll`
or `uix86.dll` exists anywhere in the SP build.

### `vmachine.h` — SP's fake-VM shim (the mechanism DEC-07 preserves)

`oracle/oracle/code/client/vmachine.h:37-92`:

```
47  struct vm_s {
48      int  (*entryPoint)( int callNum, ... );
49  };
50  typedef struct vm_s vm_t;
52  extern  vm_t  cgvm;   // interface to cgame dll or vm
53  extern  vm_t  uivm;   // interface to ui dll or vm
55  extern int VM_Call( int callnum, ... );
56  extern int VM_DllSyscall( int arg, ... );
72  extern void *Sys_LoadCgame( int (**entryPoint)(int, ...), int (*systemcalls)(int, ...) );
74  inline void *VM_Create( const char *module)
75  {
76      void *res;
77      if (!Q_stricmp("cl", module))
78      {
79          res = Sys_LoadCgame( &cgvm.entryPoint, VM_DllSyscall );
80          if ( !res)  return 0;
81      }
82      else  res = 0;
83      return res;
84  }
```

Note this `vm_t`/`VM_Create` is a **completely separate, much thinner**
type/function than MP's `qcommon/vm.cpp` versions — no `vmTable`, no
`dllHandle`/`dataBase`/`dataMask`, just a bare `entryPoint` and a hardcoded
special-case for module name `"cl"`. This is the "fake-VM shim" DEC-07 names
(`oracle/oracle/code/client/vmachine.cpp` is the matching 39-line `.cpp`, the
implementation of `VM_Call`/`VM_DllSyscall` dispatch against `cgvm`/`uivm`).

### Console/static build — `code/win32/win_main_console.cpp:531-568`

```
541  #ifndef _JK2MP
542  void *Sys_GetGameAPI (void *parms)
543  {
544      extern game_export_t *GetGameAPI( game_import_t *import );
545      return GetGameAPI((game_import_t *)parms);
546  }
547  #endif
557  #ifndef _JK2MP
558  void * Sys_LoadCgame( int (**entryPoint)(int, ...), int (*systemcalls)(int, ...) )
559  {
561      extern void cg_dllEntry( int (*syscallptr)( int arg,... ) );
562      extern int vmMain( int command, int arg0, ..., int arg7 );
563      cg_dllEntry(systemcalls);
564      *entryPoint = (int (*)(int,...))vmMain;
566      return 0;
567  }
568  #endif
```

No `LoadLibrary`/`GetProcAddress` at all — `GetGameAPI` and
`vmMain`/`cg_dllEntry` are called as plain `extern` C functions: fully
statically linked. `code/x_game/x_game.vcproj` (`ConfigurationType="4"`,
static lib, output `x_game.lib`) compiles both `game/g_main.cpp` and
`cgame/cg_main.cpp` into one static lib, linked into `code/x_exe/x_exe.vcproj`
(`ConfigurationType="1"`, exe, outputs `ja-final.exe`/`ja-release.exe`/
`ja-debug.exe`), which also compiles `ui_main.cpp`/`ui_shared.cpp` etc.
directly — game + cgame + ui all baked into one binary, zero DLL loading.

### Mac path — `oracle/oracle/code/mac/mac_main.c:35-73`

```
35  void  Sys_UnloadGame (void) {}
37  void  *Sys_GetGameAPI (void *parms) {
38      void *GetGameAPI (void *import);
39      // we are hard-linked in, so no need to load anything
40      return GetGameAPI (parms);
41  }
43  void  Sys_UnloadUI (void) {}
45  void  *Sys_GetUIAPI (void) {
46      void *GetUIAPI (void);
47      // we are hard-linked in, so no need to load anything
48      return GetUIAPI ();
49  }
65  void *Sys_LoadDll( const char *name, int (**entryPoint)(int, ...),
66      int (*systemCalls)(int, ...) ) {
68      dllEntry( systemCalls );
70      *entryPoint = vmMain;
72      return (void *)1;
73  }
```

Fully static like the console build, explicit "hard-linked in" comments at
lines 39 and 47. (Legacy Mac OS Classic/Q3-lineage file — its own top-of-file
TODO at lines 8-18 lists "dynamic loading of server game" as unimplemented —
but it's the only `mac_main.c` present and the pattern is unambiguous.)

### Conclusion (resolves DEC-07 precisely)

Not simply "retail = DLL, console = static." More specific:

- **`jagamex86.dll` is the one and only loadable module in SP, in every build
  variant.** Loaded via `LoadLibrary`/`GetProcAddress(...,"GetGameAPI")` only
  in retail (`win_main.cpp`); console/x_exe and Mac paths statically link the
  equivalent `GetGameAPI` symbol.
- **cgame is never a separate DLL, in any SP build.** In retail, cgame's
  implementation is compiled *into* `jagamex86.dll` alongside game logic, and
  the client fetches its `vmMain`/`dllEntry` via a second `GetProcAddress` on
  the *same* handle (`sv_game.cpp:676-679` → `vmachine.h:78` →
  `win_main.cpp:557-570`). In console/Mac builds cgame is compiled into the
  executable and called directly, no library step (`win_main_console.cpp:
  558-567`, `mac_main.c:65-73`).
- **UI is always fully statically linked directly into the executable, in
  every SP build variant** — no `ui.vcproj`/UI DLL project exists anywhere in
  SP.
- **Precise wording for the design doc:** "SP static via shim" is accurate
  for UI unconditionally, and for cgame in the non-retail builds; in retail
  specifically cgame is dynamically *resolved* but piggybacks on the game
  DLL's already-open handle rather than being its own module — worth stating
  precisely rather than flattened to "cgame is a DLL in retail."

---

## 5. OpenJK divergences (`tools/closure-prototype/NOTES.md`)

- **v2 / OpenJK profile** (`NOTES.md:51-68`): tool added `--source openjk
  --root <checkout>` support (its own CMakeLists flags: `_GAME`/`_CGAME`/
  `UI_BUILD`/`SP_GAME`, `-std=c++11` for SP; `NOTES.md:56-59`). Findings
  (`NOTES.md:60-68`) — OpenJK diverges from Raven 1.01 in **game-private
  structs only**:
  - `gclient_s`: OpenJK 7432 B vs oracle 7344 B.
  - `clientPersistant_t`: OpenJK 360 B vs oracle 156 B.
  - `clientSession_t`: OpenJK 164 B vs oracle 284 B.
  - saber types moved from `q_shared.h` to `bg_public.h` in OpenJK.
  - **ABI-crossing structs are unchanged**: `playerState_t` (1552 B),
    `usercmd_t` (28 B), `saberInfo_t` (2156 B), `gameState_t`.
  - Consequence stated in NOTES.md (`:65-68`): if Rust modules are ever
    loaded under OpenJK, game-private layout drift vs oracle is fine (those
    types are opaque across the seam, sized only), but any struct the
    **engine itself** dereferences must be checked against the actual host
    binary, not just the oracle — hence per-tree checking.
- **v5 / verified badges** (`NOTES.md:101-122`): badge-verification against
  OpenJK flagged `clientPersistant_t`/`clientSession_t`/`gentity_t` as `✗ SIZE
  MISMATCH` vs the oracle-derived Rust asserts — confirmed expected/correct
  ("OpenJK diverged"), not a bug (`NOTES.md:114-116`).
- **v10 / Wave 5** (`NOTES.md:270-274`): OpenJK's CMakeLists `source_groups`
  were used as a placement oracle for crate/module boundaries (engine
  common/botlib/ghoul2/icarus/server groups map ~1:1 to jka-rust crates) —
  organizational precedent, not an ABI-layout divergence.
- **No divergence claims exist for trap numbers, syscall table shape, vmMain
  signature, or dllEntry contract** — NOTES.md explicitly states the
  QVM-ABI-seam structs are unchanged between Raven and OpenJK. NOTES.md does
  not point to other files in `tools/closure-prototype/` as separate
  divergence evidence — the tooling (`closure.py`/`sweep.py`/`portpacket.py`)
  produced these numbers but isn't itself additional documentation.

**Implication for DEC-05 item 2** (loading under real OpenJK): the ABI seam
(module entrypoints, `playerState_t`/`usercmd_t`/`gameState_t`) is safe to
target directly against the oracle; game-private struct layout is **not**
guaranteed identical and must be verified per-host-build if the Rust module
is ever loaded by an actual OpenJK binary rather than retail 1.01.

---

## 6. Current Rust state (crates/)

### The four cdylib module shells

All four are thin, nearly-identical, and **not yet wired to any logic
crate**:

| Crate | `Cargo.toml` | `src/lib.rs` |
|---|---|---|
| `crates/jampgame` | `crate-type = ["cdylib"]`, dep: `abi_transport` only (`Cargo.toml:1-11`) | `pub use abi_transport::entrypoints::qvm::{dllEntry, vmMain, GetModuleAPI};` (`lib.rs:1`) |
| `crates/cgame` | same shape | same re-export |
| `crates/ui` | same shape | same re-export |
| `crates/jagame` | `crate-type = ["cdylib"]`, dep: `abi_transport` only | `pub use abi_transport::entrypoints::sp_game::GetGameAPI;` (`lib.rs:1`) |

**Important gap**: none of these four depend on their logic-tier
counterparts (`mp_game`, `mp_cgame`, `mp_ui`, `sp_game`), which live
separately at `crates/mp/game`, `crates/mp/cgame`, `crates/mp/ui`,
`crates/sp/game`. Each logic crate is *also* `crate-type = ["cdylib",
"rlib"]` (e.g. `crates/mp/game/Cargo.toml:7`) and depends on
`mp_qshared`/`mp_bg`/`mp_abi` (or `sp_*`), but a grep for
`no_mangle|vmMain|dllEntry|GetGameAPI|GetModuleAPI` across all four hits zero
— they carry the ported data model (`crates/mp/game/src/lib.rs:1-14`: `ai,
botai, client, entity, level, npc, saber, say, teams` modules,
`//TODO: Port the gameplay logic (g_*.c functions)` at `:3-5`) but export no
entrypoints. **Today there are two unconnected module-crate lineages**: the
exported-symbol shells (top-level `crates/jampgame` etc.) and the logic
crates (`crates/mp/game` etc.) — the design doc needs to specify how/whether
these merge.

### `crates/abi-transport/src/entrypoints.rs`

- Raw ABI type aliases: `RawSyscall`, `RawImportTable`, `RawExportTable`,
  `RawDllEntry`, `RawVmMain`, `RawGetModuleApi`, `RawGetGameApi`
  (`entrypoints.rs:1-27`).
- `pub mod qvm` (`entrypoints.rs:29-65`), three `#[no_mangle] extern "C"`
  stubs:
  - `dllEntry(_syscall: RawSyscall)` — empty body (`:33-34`).
  - `vmMain(_command: AbiCommand, _arg0..._arg11: AbiWord) -> AbiWord` —
    12-word signature, returns `0` (`:37-55`).
  - `GetModuleAPI(_api_version: AbiCommand, _import: RawImportTable) ->
    RawExportTable` — returns `null_mut()` (`:58-64`).
- `pub mod sp_game` (`entrypoints.rs:67-75`): `GetGameAPI(_import:
  RawImportTable) -> RawExportTable` — returns `null_mut()` (`:71-74`).
- All four are literal no-op stubs — no dispatch logic yet.
- `crates/abi-transport/src/lib.rs:1-8`: scope note — "cross-mode ABI
  transport... No Raven game types cross here — only the wire transport."
  Supporting plumbing: `generic/table.rs` (`FunctionTableImport`/
  `FunctionTableExport` traits, `:4-14`), `transport/syscall.rs`
  (`SysCallTransport`/`EncodeSysCall`, word-packing only, `:1-30`),
  `transport/vm_main.rs` (same for vmMain words) — typed encode/decode
  helpers, no loader.

### `crates/native/platform`

**Nothing exists yet for dylib loading.** Exactly two files:
- `src/lib.rs:1-5` — `pub mod platform;`, doc comment "Raven-free OS/platform
  primitives (cross-mode)."
- `src/platform.rs:1-18` — only Windows-compat type aliases (`LPCTSTR`,
  `LPCSTR`, `DWORD`, `UINT`, `HANDLE`, `COLORREF`, `BYTE`) ported from
  `oracle/oracle/codemp/qcommon/platform.h:13-20`.

Repo-wide grep for `libloading|dlopen|LoadLibrary|dylib.*load` across all
`*.rs`/`*.toml` (excluding `oracle/oracle/`) returns **zero matches**. No
`libloading` dependency exists anywhere in the workspace; no
dlopen/LoadLibrary wrapper exists at any path yet.

### Docs already recording intent

- `docs/workspace-architecture.md:64-67,75-76`: `mp/abi` = "MP engine<->module
  seam (dllEntry/vmMain surfaces)"; `mp/game`/`cgame`/`ui` = the module
  cdylibs; `sp/abi` = "SP: GetGameAPI table (game) + dllEntry/vmMain
  (cgame/ui)"; `sp/game` = "jagame (cdylib, GetGameAPI table ABI)".
- `docs/workspace-architecture.md:133`: migration mapping —
  `src/abi/entrypoints.rs` → "each module cdylib `lib.rs`" —
  `dllEntry`/`vmMain`/`GetModuleAPI`/`GetGameAPI`.
- `docs/workspace-architecture.md:163-166`: open item — SP cgame/ui transport
  "resolved per `decisions.md` DEC-07: statically linked into `sp/app` behind
  the vmachine shim... (design in `docs/architecture/module-loading.md`)."
  **`docs/architecture/module-loading.md` does not exist** — confirmed
  neither the file nor `docs/architecture/` exists in the repo. This dossier
  (and the design session it feeds) is the forward reference finally being
  resolved.
- `docs/decisions.md:49-70` — DEC-05 (quoted in full in §8 below).
- `docs/decisions.md:79-85` — DEC-07 (quoted in full in §4 above /
  reproduced in §8).
- `docs/type-port-todo.md` has no mentions of module-loading/cdylib/
  dllEntry/vmMain/libloading — purely a type-port status ledger, silent on
  this topic.

---

## 7. WASM transport constraints (DEC-05.5)

From the ABI surface actually in the oracle and in `abi-transport` today:

- **vmMain 12-word export analog**: Raven's `vmMain(int command, int arg0,
  ..., int arg11)` (`game/g_main.c:515` et al., §1 above) is already
  mirrored 1:1 in Rust as `vmMain(_command: AbiCommand, _arg0..._arg11:
  AbiWord) -> AbiWord` (`crates/abi-transport/src/entrypoints.rs:37-55`) — a
  wasm32 export with the same 13-word signature (all `i32`) is a direct,
  mechanical translation; no shape change needed, only a real body.
- **syscall import**: Raven's `dllEntry(syscall)` handshake
  (`win_main.cpp:879-885`, §1) hands the module a function pointer; wasm
  can't pass function pointers across the module boundary the same way — the
  wasm module would need to **import** a `syscall(argsPtr: i32, argc: i32) ->
  i32`-shaped host function (or similar) at instantiation time rather than
  receiving a callback pointer via `dllEntry`. `abi-transport`'s existing
  `SysCallTransport`/`EncodeSysCall` (`transport/syscall.rs:1-30`) already
  models syscalls as word-encoded messages rather than raw pointer-bearing
  calls, which is compatible with a wasm import — but nothing there yet
  targets wasm specifically (it's transport-agnostic word packing, used
  today only by the native path).
- **VM_ArgPtr-style linear-memory translation precedent**: exactly the QVM
  masked-pointer scheme in §1's last section (`vm.cpp:640-654`,
  `dataBase + (intValue & dataMask)`) — this is the direct precedent DEC-05.5
  names. A wasm32 module's linear memory is the `dataBase` analog; any
  "pointer" a wasm module hands to a syscall is really an `i32` offset that
  the host must translate via `memory.data(offset)` plus an explicit bounds
  check (wasmtime/wasm-bindgen equivalent of the `& dataMask` fence) —
  **not** a raw host pointer, ever.
- **Trap categories that can't pass raw pointers**: any trap whose C
  signature takes/returns a pointer to engine-owned or module-owned memory
  (e.g. string returns, `trace_t*`/`playerState_t*` out-params, callback
  function pointers like `dllEntry`'s syscall arg) must be re-expressed as
  (linear-memory offset + length) pairs or copy-in/copy-out through a shared
  buffer, for a wasm target — this is every syscall in the game/cgame/ui
  import tables that isn't a plain scalar in/out. The native-DLL and QVM
  transports both currently get away with raw pointers (native: fully
  trusted, §1; QVM: masked-but-still-a-real-host-pointer-after-translation,
  §1) — wasm is the first transport where the pointer must **never** leave
  the sandbox as a real address.
- **What a wasm32 build check on module crates entails today** (no builds
  run, per instructions — dependency inspection only):
  - `crates/native/platform` currently only holds Windows type aliases
    (`HANDLE`, `COLORREF`, etc., `platform.rs:1-18`) — Windows-API-flavored
    code in a crate module crates might transitively depend on is an
    immediate wasm32 red flag if module crates ever pull it in for
    non-cfg-gated reasons.
  - No `libloading`/`dlopen` dependency exists yet (confirmed in §6), so no
    red flag there *yet* — but this is exactly the dependency a native dylib
    loader crate would need, and it must stay **out of** the module crates
    (`mp_game`/`mp_cgame`/`mp_ui`/`sp_game`) and confined to the *host* side
    (`native/platform` or an engine crate), or wasm32 builds of the module
    crates will fail to compile.
  - `qshared`/`bg` crates were not directly re-inspected in this pass for
    `libc` usage; the task brief flags `libc` usage in qshared/bg as the
    general category of red flag to watch for (any direct libc:: syscalls,
    threading, or filesystem calls in code that must also compile for
    `wasm32-unknown-unknown`/`wasm32-wasi`) — worth a dedicated grep
    (`grep -rn "libc::" crates/*/qshared crates/*/bg`) before committing to
    "module crates build clean for wasm32" as a design assumption.

---

## 8. Symbol/ABI details for real-engine loading (consolidated)

- **Calling convention**: `QDECL` = `__cdecl` on Windows
  (`oracle/oracle/codemp/game/q_shared.h:139,152`) — caller-cleans-stack,
  which is why `VM_Call`'s over-wide 16-arg native invocation
  (`vm.cpp:816-819`) against a 13-int-arity callee is harmless on x86/cdecl
  but is not a portable assumption for other ABIs/architectures.
- **32-bit int/pointer assumptions**: `vmMain`'s entire signature is `int`
  words (`game/g_main.c:515` et al.); `vmptr_t` is `typedef int vmptr_t`
  (`vm_local.h:99`); `dataBase`/`dataMask` are 32-bit (`vm_local.h:135-136`).
  This is exactly why DEC-05 item 3 gates real-DLL hosting to
  `i686-pc-windows` only — the ABI is baked-in 32-bit throughout, not just at
  the entrypoint.
- **GOAL.md checklist**: (not separately re-read in this pass beyond what's
  cited in workspace-architecture.md/decisions.md above — flag for the design
  session to cross-check GOAL.md's checklist items against DEC-05/DEC-07
  directly if the design doc needs a checklist-item citation.)
- **Decision-ledger text, verbatim** (`docs/decisions.md`):
  - **DEC-05** (`:49-70`): scope includes (1) Rust engine ↔ Rust modules via
    `native/platform` dylib loading honoring Raven's entry symbols; (2) Rust
    modules loaded by real engines — retail `jamp` 1.01 (`i686-pc-windows`
    module builds) and OpenJK native builds (accounting for §5's
    divergences); (3) Rust engine hosting real/mod DLLs
    (`jampgamex86.dll` replacements: JA+, MBII) — **`i686-pc-windows` engine
    build only**, since retail DLLs are 32-bit Windows PE; (4) classic QVM
    bytecode interpreter is **out of scope**; (5) **WASM module transport is
    a first-class target from the start** — pluggable transport
    `NativeDll | Static | Wasm`, `architecture/engine-seam.md` +
    `architecture/module-loading.md` (neither written yet) must design the
    WASM variant explicitly (linear-memory pointer translation à la
    `VM_ArgPtr`, handle-only trap surface, 32-bit in-module layouts); module
    crates get `wasm32` build checks in CI from the beginning; the wasmtime
    host itself lands **after** native-DLL parity is proven.
  - **DEC-07** (`:79-85`): `sp/app` statically links `sp/cgame` + `sp/ui`;
    Raven's fake-VM shim (`oracle/oracle/code/client/vmachine.cpp`) survives
    as a thin dispatch layer preserving the `VM_Call` ABI shape, matching
    shipped `jasp`. The retail load-from-DLL variant is **not** ported.
    Resolves workspace-architecture's "SP transport" open item.

---

## Design forks

Open questions for the design session, informed by the ground truth above:

1. **Transport trait shape** — enum (`NativeDll | Static | Wasm` as DEC-05
   literally names it) vs. trait object vs. compile-time generic per module.
   The existing `abi-transport` crate already has a transport-agnostic
   word-encoding layer (`SysCallTransport`/`EncodeSysCall`,
   `transport/syscall.rs`) that doesn't commit to a dispatch mechanism yet —
   decide whether that stays a trait (`ModuleTransport::call(...)`) with
   per-variant impls, or whether the three variants are different enough
   (native: real function pointer + `libloading`; static: direct Rust call;
   wasm: instance + linear-memory translation) that a flat enum-match at the
   call site is more honest than forcing a shared trait.
2. **Where filename/search-path policy lives** — Raven scatters this
   per-platform (`win_main.cpp:826` "%sx86.dll", `unix_main.c:341-356` arch
   `#ifdef` ladder, no shared helper anywhere in oracle, §3). Rust should
   almost certainly centralize this in `native/platform` (currently empty,
   §6) as a single policy function, rather than reproducing Raven's
   per-platform duplication — but the search-order semantics (fs_game →
   basepath → cdpath for MP §1; the much narrower cwd-relative search for SP
   §4) differ enough between MP and SP that the policy object probably needs
   mode-specific configuration, not a single hardcoded order.
3. **VM restart semantics in Rust** — Raven's own `VM_Restart` (§1) already
   demonstrates two genuinely different restart strategies (native:
   destroy+recreate; QVM: in-place data-segment reset) baked into one
   function via a boolean-like branch on `dllHandle`. Since QVM/bytecode is
   explicitly out of scope (DEC-05 item 4), Rust's native/static/wasm
   transports may not need an in-place-restart mode at all — decide whether
   "map_restart" in Rust is always drop+reload (simpler, matches what native
   DLLs actually did in Raven) or whether a wasm instance's cheap
   re-instantiation cost justifies a genuine in-place-reset fast path
   analogous to QVM's (wasm linear memory can be memset/reset without
   recreating the whole `Instance`, unlike a native DLL).
4. **wasm pointer-translation strategy** — directly modeled on `VM_ArgPtr`'s
   masked-offset scheme (§1, §7): decide the concrete Rust shape (a
   `WasmPtr<T>` newtype over `u32` + a translation trait bound to a
   `wasmtime::Memory` or equivalent, bounds-checked rather than
   power-of-2-masked since wasm memories aren't guaranteed power-of-2 sized)
   and which trap signatures must be rewritten to (offset,len) pairs instead
   of raw pointers (§7's trap-category list) — this rewrite touches every
   import/export table, not just the entrypoints, so scope it explicitly
   against `mp/abi`'s existing table definitions.
5. **Does the jampgame cdylib double as the wasm artifact source?** — today
   `crates/jampgame` is a bare re-export shim over `abi-transport`
   (`lib.rs:1`, §6) and doesn't even depend on the logic crate
   (`crates/mp/game`) yet. Decide whether the eventual wired-up
   `crates/jampgame` crate is compiled twice with different `--target`
   (`x86_64`/`i686` cdylib vs `wasm32` cdylib) from one source, or whether
   wasm needs a separate crate wrapping the same logic crate with a
   wasm-specific entrypoint module (parallel to how `abi-transport::
   entrypoints::qvm` vs a hypothetical `entrypoints::wasm` might split) —
   this decision also determines whether `native/platform`-style
   Windows-only code (§7's red flag) can ever leak into the shared logic
   crate or must be strictly confined to the native-only entrypoint shim.
6. **Shared-global footguns to deliberately not port** — `VM_Free`'s
   unconditional clobber of the *global* `currentVM`/`lastVM` regardless of
   which VM was freed (`vm.cpp:624-625`, §1) is a Raven bug/quirk, not a
   contract; the Rust module registry (whatever replaces `vmTable[MAX_VM]`,
   §1) should make "current module" state per-slot, not global, from day
   one — flag this explicitly so the design doc doesn't accidentally
   re-introduce it for faithfulness's sake (porting-rules §A2 allows
   deviation here since it's host-side bookkeeping, not an ABI-crossing
   struct).
7. **SP shim scope precision** — DEC-07 says "SP static via shim," but §4
   shows the *actual* Raven behavior is retail-cgame-piggybacks-on-jagame-DLL
   vs. console/Mac-fully-static; since DEC-07 explicitly says "the retail
   load-from-DLL variant is not ported," confirm the design doc states
   plainly that jka-rust's SP always uses the console/Mac-style fully-static
   shape (`vmachine.h`/`vmachine.cpp` preserved as dispatch, no
   `Sys_LoadCgame`/`GetProcAddress` ever exercised) — i.e. DEC-07 already
   answers this, but the dossier's job is making sure the design doc doesn't
   conflate "shim" with "still dynamically loads."
