# B2 — Filesystem: Ground-Truth Dossier

Scope: the virtual filesystem (search paths, pak files, handles) — MP
`qcommon/files_common.cpp`+`files_pc.cpp`, SP `code/qcommon/files_common.cpp`+
`files_pc.cpp`. Every claim cites `oracle/<path>:<line>`. Builds on
`docs/dossiers/A2-state-ownership.md` §1d (FS global census).

Status: complete.

---

## 0. Build-canonical verification

**MP**: confirmed via `oracle/codemp/jk2mp.vcproj:1466,1497`,
`oracle/codemp/WinDed.vcproj:219,222`, and
`oracle/codemp/unix/makefile:305-307,396,398` (the `ded` target) — all
three real MP build targets (client, dedicated Windows, dedicated Linux) link
`files_common.cpp` + `files_pc.cpp`. `files.cpp` (a single-TU merge of both,
used only by the `q3static` unix target at `unix/makefile:933,1114`) and
`files_console.cpp` (Xbox, `x_exe/x_exe.vcproj:316-319`) are dead for the PC
build — consistent with A2 and the CLAUDE.md caveat. `unix/files_linux.cpp` is
a thin platform shim, not a third variant of the FS core.

**SP**: `oracle/code/starwars.vcproj:1000,1027` (the real PC exe
project) links `files_common.cpp` + `files_pc.cpp` — the **same pairing as
MP**. `files_console.cpp` (1033 lines, `oracle/code/qcommon/`) is
Xbox-only per `oracle/code/x_exe/x_exe.vcproj:264,267` (which pairs
`files_common.cpp` + `files_console.cpp`, no `files_pc.cpp`). So for SP PC,
`files_console.cpp` is the dead variant, not `files.cpp` (SP has no top-level
`files.cpp` at all — only MP does). **Correction to task framing**: "files.cpp
is a dead console variant" is true for MP; for SP the dead file is
`files_console.cpp` (name differs, role identical).

Line counts: MP `files_common.cpp` 512 / `files_pc.cpp` 3130. SP
`files_common.cpp` 588 / `files_pc.cpp` 1741 (SP is materially smaller —
no pure-server code, see §6).

---

## 1. Search-path model

### 1a. Global state (builds on A2 §1d)

All declared in `oracle/codemp/qcommon/files_common.cpp:183-224`,
extern'd in `oracle/codemp/qcommon/files.h:103-143`:

- `fs_gamedir[MAX_OSPATH]` — current gamedir name only (no separators), L183.
- Cvars `fs_debug, fs_homepath, fs_basepath, fs_basegame, fs_cdpath,
  fs_copyfiles, fs_gamedirvar, fs_restrict, fs_dirbeforepak` — L184-192, all
  registered in `FS_Startup` (`files_pc.cpp:2491-2504`).
- `fs_searchpaths` (linked-list head) — L193.
- `fs_readCount/fs_loadCount/fs_loadStack/fs_packFiles` — L194-197.
- `fs_fakeChkSum/fs_checksumFeed` — L199-200.
- `fsh[MAX_FILE_HANDLES]` — L202 (size correction, see §3).
- Pure-server-only: `fs_numServerPaks/fs_serverPaks[4096]/fs_serverPakNames[]`,
  `fs_numServerReferencedPaks/fs_serverReferencedPaks[]/
  fs_serverReferencedPakNames[]` — L206-214 (see §4).
- `lastValidBase/lastValidGame[MAX_OSPATH]` — L217-218 (used only by
  `FS_Restart`'s MP-only fallback path, §4).
- `initialized` — L224.

### 1b. FS_Startup order (`files_pc.cpp:2483-2576`)

Cvars registered first (L2491-2504), then search paths added in **ascending
priority order** (each `FS_AddGameDirectory` call prepends to
`fs_searchpaths`, so later calls end up searched first):

1. `fs_cdpath` + `gameName`, if `fs_cdpath` non-empty (L2507-2509).
2. `fs_basepath` + `gameName` (L2510-2512).
3. `fs_homepath` + `gameName`, only if `fs_homepath != fs_basepath`
   (case-insensitive) — "somewhat particular to *nix systems" (L2513-2517).
4. If `fs_basegame` set and `gameName == BASEGAME` and `fs_basegame !=
   gameName`: repeat steps 1-3 for `fs_basegame` (L2519-2530) — lets a mod
   base itself on another mod's assets.
5. If `fs_gamedirvar` (`fs_game`) set and `gameName == BASEGAME` and
   `fs_gamedirvar != gameName`: repeat steps 1-3 for `fs_gamedirvar`
   (L2532-2543) — the actual "mod folder" search paths.

Net effect: search order (highest priority first) is `fs_game` dir >
`fs_basegame` dir > `base`/`gameName` dir, and within each, homepath >
basepath > cdpath. After paths are added: `FS_ReorderPurePaks()` runs
(L2561, see §4), then `FS_Path_f()` prints the final list (L2564,
`files_pc.cpp:2140-2166`).

`FS_Startup` is called twice in practice: once with `DEMOGAME` from
`FS_SetRestrictions` (L2626) and once with `BASEGAME` from `FS_Restart`
(L3000).

SP's `FS_Startup` (`code/qcommon/files_pc.cpp:1550-1583`) registers only
`fs_debug, fs_copyfiles, fs_cdpath, fs_basepath, fs_gamedirvar, fs_restrict`
(no `fs_homepath`, `fs_basegame`, `fs_dirbeforepak`) and adds paths in the
same relative order (cdpath, basepath, then mod dir) but with no homepath
step and no basegame-of-basegame step — confirms A2's "narrower, no
pure-server-adjacent cvars" note and extends it: SP also lacks the
mod-basing-on-mod feature entirely.

### 1c. AddGameDirectory pak discovery/sort (`files_pc.cpp:2211-2294`)

- Dedup guard: if a `searchpath_t` already has this exact `(path, dir)` pair
  (case-insensitive), return without re-adding (L2225-2229) — "fixes the case
  where fs_basepath is the same as fs_cdpath."
- The directory node itself is unconditionally prepended to
  `fs_searchpaths` (L2241-2242) **before** pak discovery.
- `Sys_ListFiles(pakfile, ".pk3", NULL, &numfiles, qfalse)` lists all `.pk3`
  in the directory (L2250); capped at `MAX_PAKFILES = 1024` (L2211,2254-2256).
- **Sort rule, cited exactly**: `qsort(sorted, numfiles, 4, paksort)` where
  `paksort` calls `FS_PathCmp` (ordinal, case/separator-insensitive string
  compare) (L2261, `paksort` at L2194-2201, `FS_PathCmp` at L2041-2071).
  Comment: "sort them so that later alphabetic matches override earlier
  ones. This makes pak1.pk3 override pak0.pk3" (L2252-2253). So paks are
  loaded in ascending alphabetical order, and since each successfully-loaded
  pak's `searchpath_t` node is **prepended** to `fs_searchpaths` (or spliced
  in right after the directory node under `fs_dirbeforepak`, see below), the
  **last-loaded (alphabetically highest, e.g. `assets2`/`pak9`) pak ends up
  searched first**. This is the mechanism behind "higher-numbered pak wins."
- `fs_dirbeforepak` (L2273-2284): if set, each new pak node is spliced in
  right after the directory node (`thedir->next`), preserving pak-vs-pak
  alphabetical-descending order within that splice, but keeping the
  directory itself ahead of the game's own paks. If unset (default, `"0"`
  at L2504), paks are prepended to the *global* `fs_searchpaths` head,
  meaning a later `FS_AddGameDirectory` call's paks (and even its raw
  directory) can outrank an earlier call's paks — i.e. mod-dir paks always
  outrank basegame paks regardless of this cvar, but the cvar controls
  directory-vs-own-pak ordering within one `FS_AddGameDirectory` call.

SP's `FS_AddGameDirectory` (`code/qcommon/files_pc.cpp:1475-1552`) uses the
identical dedup guard, `Sys_ListFiles(...,".pk3",...)` + `qsort` +
`FS_PathCmp` sort, and prepend-to-head insertion — but has **no
`fs_dirbeforepak` branch at all** (that whole conditional is absent); packs
are unconditionally prepended to `fs_searchpaths` (L1531-1536).

---

## 2. Pak (zip) reading

- Vendored minizip fork: `oracle/codemp/qcommon/unzip.{h,cpp}` (1337
  + 289 lines) and sibling `zlib32/`. SP has its own copy at
  `oracle/code/qcommon/unzip.{h,cpp}`. Per project policy these become
  Rust `zip`/`flate2` crate calls at the seam (see Design forks).
- Entry points actually called from `files_pc.cpp`: `unzOpen`,
  `unzGetGlobalInfo`, `unzGoToFirstFile`/`unzGoToNextFile`,
  `unzGetCurrentFileInfo`, `unzGetCurrentFileInfoPosition`,
  `unzSetCurrentFileInfoPosition`, `unzReOpen`, `unzOpenCurrentFile`,
  `unzCloseCurrentFile`, `unzClose`, `unzReadCurrentFile` (see `FS_Read2`
  path). Grep-confirmed call sites: `FS_LoadZipFile`
  (`files_pc.cpp:1423-1522`), `FS_FOpenFileRead`
  (`files_pc.cpp:819,832,834,838`), `FS_FCloseFile` (`files_pc.cpp:461-483`).
- **Pak checksum computation** (`FS_LoadZipFile`, `files_pc.cpp:1423-1522`):
  for every zip entry with `uncompressed_size > 0`, its `LittleLong(crc)` is
  appended to a scratch `int[]` buffer (L1497-1499); after iterating all
  entries:
  - `pack->checksum = Com_BlockChecksum(fs_headerLongs, 4*fs_numHeaderLongs)`
    (L1513) — the "regular" checksum, used for `FS_ComparePaks`
    autodownload matching (§4) and demo-pak validation
    (`FS_SetRestrictions`, L2632).
  - `pack->pure_checksum = Com_BlockChecksumKey(fs_headerLongs,
    4*fs_numHeaderLongs, LittleLong(fs_checksumFeed))` (L1514) — the
    **keyed** checksum used for pure-server validation, keyed by the
    per-connection `fs_checksumFeed` so a client can't fake pak contents by
    replaying a known checksum from a different session.
  - Both are then run through `LittleLong` again for endianness (L1515-1516).
  - SP's `FS_LoadZipFile` (`code/qcommon/files_pc.cpp:865-953`) computes
    only the unkeyed `pack->checksum` (L951) — no `pure_checksum` field
    exists on SP's `pack_t` at all (`code/qcommon/files.h:33-43`, missing
    `pakBasename`, `pakGamename`, `pure_checksum`, `referenced`).
- **`fs_checksumFeed` flow** (MP only): server picks a fresh feed value per
  map load — `sv.checksumFeed = ((rand()<<16)^rand())^Com_Milliseconds()`
  (`server/sv_init.cpp:626`) — then calls `FS_Restart(sv.checksumFeed)`
  (L627), which stores it in the global `fs_checksumFeed`
  (`files_pc.cpp:2994`) before any paks are (re)loaded, so every pak's
  `pure_checksum` for that session is keyed consistently. The feed is sent
  to the client in `SV_SendClientGameState`
  (`server/sv_client.cpp:765,1407,1700`) and echoed back by the client
  (`client/cl_main.cpp:406`); on receipt the client stores it as
  `clc.checksumFeed` (`client/cl_parse.cpp:636`) and calls
  `FS_ConditionalRestart(clc.checksumFeed)` (`cl_parse.cpp:649`;
  also `cl_main.cpp:1332`), which only actually restarts the FS (and thus
  re-keys local paks' `pure_checksum`) if the feed or `fs_gamedirvar`
  changed (`files_pc.cpp:3048-3054`).

---

## 3. Handle table

**`fsh` size correction to A2**: A2 §1d cites `fsh[MAX_FILE_HANDLES=16]` for
MP. That constant is conditional:
`oracle/codemp/qcommon/qcommon.h:507-511` —
`#ifdef _XBOX` → 16, `#else` → **64**. The PC build (the build-canonical one
per §0) is non-Xbox, so **MP's real `fsh` array is `fileHandleData_t[64]`**,
not 16. SP has no such conditional: `oracle/code/qcommon/files.h:58`
hardcodes `MAX_FILE_HANDLES 16` unconditionally — so SP really is 16. Net:
**MP=64, SP=16**, not "16" for both as the task's shorthand `fsh[16]`
suggested.

- `fileHandleData_t` layout (MP: `files.h:84-100`; SP: `code/qcommon/
  files.h:72-87`): `qfile_ut handleFiles` (union of `FILE*`/`unzFile`, plus a
  `unique` bool), `handleSync`, `baseOffset`, `fileSize`, `zipFilePos`,
  `zipFile`, `streamed` (MP only — SP lacks this field), `name[MAX_ZPATH]`
  (MP) / `name[MAX_QPATH]` (SP, smaller: 64 vs 256 bytes).
- `FS_HandleForFile` (`files_common.cpp:258-268`): linear scan `i=1..
  MAX_FILE_HANDLES-1` for the first slot with `handleFiles.file.o == NULL`;
  fatal `Com_Error(ERR_DROP, ...)` if none free. **Handle 0 is never
  issued** — it's the sentinel for "no file"/failure throughout the API
  (e.g. `FS_FOpenFileRead` returns `*file = 0` on failure, L708,715,995).
- **`FS_FOpenFileRead` resolution order** (`files_pc.cpp:672-997`,
  exact walk): reject paths containing `..` or `::` (L707-710); reject
  `q3key` access after full init (L714-717); allocate a handle up front
  (L723); then **for each `searchpath_t` node head-to-tail** (i.e. in
  `fs_searchpaths` list order, which is exactly the priority order built in
  §1):
  1. If the node is a pack (`search->pack`): hash the filename into the
     pack's table (L740), and if a bucket hit exists, walk the pak's pure
     check (`FS_PakIsPure`, L745-747, §4) and then the hash-chain doing
     case/separator-insensitive `FS_FilenameCompare` (L754) — first match
     wins, sets reference flags (L757-815), opens via `unzReOpen` (unique)
     or reuses `pak->handle` (shared) (L817-825), returns
     `uncompressed_size` (L854).
  2. Else if the node is a directory (`search->dir`): if
     `fs_restrict` or `fs_numServerPaks` is set, only `.cfg/.fcf/.menu/
     .game/.dm_<PROTOCOL>/.dat` extensions are allowed from directories
     (L869-879); build the OS path and `fopen(..., "rb")` (L883-887); on
     success, optionally trigger `fs_copyfiles` cache-sync behavior
     (L897-967, Windows-only, `#ifndef __linux__`); returns
     `FS_filelength(*file)` (L983).
  3. If neither matched, continue to `search->next`.
  - The whole per-searchpath loop is wrapped in a `do {...} while
    (bFasterToReOpenUsingNewLocalFile)` outer loop solely to support the
    `fs_copyfiles==2` local-cache-invalidation re-open path (L733-987) — not
    part of the logical resolution order.
  - Not found: `*file = 0; return -1` (L995-996).
- **`FS_ReadFile`/`FS_FreeFile`** (`files_pc.cpp:1259-1405`): despite the
  function names/comments referencing the hunk temp-memory allocator, the
  **live MP code path uses `Z_Malloc`/`Z_Free`, not
  `Hunk_AllocateTempMemory`/`Hunk_FreeTempMemory`** — the hunk-based
  implementation is present but fully commented out (L1347-1350 vs the live
  `Z_Malloc` at L1352, `FS_FreeFile`'s hunk block commented at
  L1380-1395 vs the live `Z_Free` at L1404). `fs_loadStack`/`fs_loadCount`
  bookkeeping is still incremented (L1308-1309, L1346) even though nothing
  reads `fs_loadStack` to drive hunk clearing anymore in this path — it's
  live dead-ish state, still exposed via `FS_LoadStack()`
  (`files_common.cpp:253-256`). `.cfg` files get special journal-replay
  handling when `com_journal->integer == 2` (playback) or `==1` (record) —
  L1277-1315, L1327-1341, L1364-1370.
- **Write paths**:
  - `FS_FOpenFileWrite` (`files_pc.cpp:491-524`) and `FS_FOpenFileAppend`
    (L532-564): both build the OS path via `FS_BuildOSPath(fs_homepath->
    string, fs_gamedir, filename)` — i.e. **writes always target
    homepath**, never basepath/cdpath, regardless of where the file was
    read from. `FS_CreatePath` (mkdir -p equivalent) is called first; on
    failure returns handle `0`.
  - `FS_SV_FOpenFileWrite` (L272-320) is the server-side variant (used for
    server-side journal/save files), building the path directly under
    `fs_homepath` + the raw filename (no per-gamedir insertion beyond what
    the caller passes).
  - `FS_WriteFile` (`files_common.cpp:401-421`) is the convenience
    whole-buffer wrapper: open, `FS_Write`, `FS_FCloseFile`.
  - Journal/config writing rides through the same `FS_Write`/`FS_Flush`
    calls against `com_journalDataFile` inside `FS_ReadFile`'s `.cfg`
    branch (L1327-1341,1364-1370) — there is no separate "config writer,"
    `Com_WriteConfiguration`-style callers just call `FS_FOpenFileWrite` +
    `FS_Printf` (`files_common.cpp:375-384`) like any other file.
- **`FS_FileExists` variants** (`files_pc.cpp:219-263`): `FS_FileExists`
  tests only `fs_homepath/fs_gamedir/<file>` (the write-target location) —
  explicitly documented as "DOES NOT search the paths... to determine if
  opening a file to write... will cause any overwrites" (L221-224).
  `FS_SV_FileExists` tests `fs_homepath/<file>` directly (no gamedir
  insertion, L249-263) — used for server-side pak-already-downloaded checks
  in `FS_ComparePaks` (§4).

---

## 4. Pure-server machinery (MP-only)

- **Correction to A2's flagged "bug"**: A2 §1d cites
  `files_pc.cpp:2328-2330` as a "static-shadowing bug" where the
  referenced-paks trio is re-declared `static`, shadowing the
  `files_common.cpp` globals. Verified: lines 2328-2330 are **inside the
  block comment** documenting `FS_ComparePaks` (comment spans
  `files_pc.cpp:2315-2338`; the `static int fs_numServerReferencedPaks;`
  etc. text sits at 2328-2330, all inside `/* ... */`). This is dead
  comment text, not compiled code — grep-only tools (or a stale earlier
  pass) would misread it as a real declaration. The actual, sole
  definitions are `files_common.cpp:212-214` (extern'd at
  `files.h:133-135`), and `FS_ComparePaks`
  (`files_pc.cpp:2339-2409`)/`FS_PureServerSetReferencedPaks`
  (L2947-2981) both read/write those same externs with no shadowing.
  **Effective behavior: no bug — one set of globals, consistently used.**
  (Flagging this so the design doc doesn't inherit a false bug-compat
  requirement.)
- **`FS_PureServerSetLoadedPaks`** (`files_pc.cpp:2887-2936`): tokenizes
  `pakSums` (space-separated checksum ints) into `fs_serverPaks[]`
  (capped `MAX_SEARCH_PATHS=4096`, L2893-2901); if the new count is 0 and
  `fs_reordered` was set by a prior `FS_ReorderPurePaks` call, forces a full
  `FS_Restart(fs_checksumFeed)` to fix search order (L2906-2916, referencing
  id Software bugzilla #540); otherwise re-tokenizes `pakNames` into
  `fs_serverPakNames[]`, freeing old entries first (L2918-2935).
- **`FS_PureServerSetReferencedPaks`** (L2947-2981): identical shape but
  populates `fs_numServerReferencedPaks`/`fs_serverReferencedPaks[]`/
  `fs_serverReferencedPakNames[]` — used purely for autodownload matching
  (`FS_ComparePaks`), not for restricting what the client may load.
- **`FS_PakIsPure`** (`files_pc.cpp:39-55`): if `fs_numServerPaks` is 0,
  every pak is pure (menu/offline case); else a pak is pure iff its
  (unkeyed) `checksum` matches one of `fs_serverPaks[]` — comment flags a
  known false-positive risk: two different-named pk3s with colliding
  checksums both pass (L43-46).
- **`FS_ReorderPurePaks`** (L2445-2476): walks the server's pak-checksum
  list in order and, for each, finds the matching `searchpath_t` node
  (`fs_pack->checksum == fs_serverPaks[i]`) and splices it to the front of
  `fs_searchpaths`, preserving the server's order — this is a pure
  in-place linked-list reorder, **not reflected in any cvar** (own comment,
  L2441-2442, citing bugzilla #540). Only runs if `fs_numServerPaks != 0`
  (L2453-2454); called once at the end of every `FS_Startup` (L2561).
- **`FS_ConditionalRestart`/`FS_Restart`** (`files_pc.cpp:2988-3054`):
  `FS_Restart(checksumFeed)` = full `FS_Shutdown(qfalse)` →
  `fs_checksumFeed = checksumFeed` → `FS_ClearPakReferences(0)` →
  `FS_Startup(BASEGAME)` → `FS_SetRestrictions()` → sanity-check
  `mpdefault.cfg` loads, else roll back to `lastValidBase/lastValidGame`
  and retry once, else fatal (L3005-3023) → conditionally exec
  `jampserver.cfg`/`jampconfig.cfg` if the gamedir actually changed and not
  in safe mode (L3025-3035) → snapshot `lastValidBase/lastValidGame`
  (L3037-3038). `FS_ConditionalRestart(checksumFeed)` only calls
  `FS_Restart` if `fs_gamedirvar->modified` or the feed changed
  (L3048-3054) — this is the client-side gate invoked from
  `cl_parse.cpp:649` on `svc_setgame`/gamestate.

---

## 5. Module seam

- **Trap plumbing**: `fileHandle_t` is `typedef int` (see §6 for the
  enum-vs-alias implication) and crosses the VM boundary as a plain
  integer arg/return — no marshaling. Confirmed for all three MP modules:
  - `game/g_syscalls.c:85-98,219-220` (`trap_FS_FOpenFile/Read/Write/
    FCloseFile/GetFileList` → `G_FS_FOPEN_FILE`, `G_FS_READ`, `G_FS_WRITE`,
    `G_FS_FCLOSE_FILE`, `G_FS_GETFILELIST`), handled engine-side in
    `server/sv_game.cpp:552-564` — direct calls into
    `FS_FOpenFileByMode`/`FS_Read2`/`FS_Write`/`FS_FCloseFile`/
    `FS_GetFileList`.
  - `cgame/cg_syscalls.c:83-100` (`CG_FS_FOPENFILE` etc.), handled in
    `client/cl_cgame.cpp:737` (case block).
  - `ui/ui_syscalls.c:83-100` (`UI_FS_FOPENFILE` etc.), handled in
    `client/cl_ui.cpp:914` (case block).
  - `FS_FOpenFileByMode` (`files_pc.cpp:3064-...`) is the single dispatcher
    all three module traps route through — switches on `fsMode_t` to call
    `FS_FOpenFileRead`(`FS_READ`)/`FS_FOpenFileWrite`(`FS_WRITE`)/
    `FS_FOpenFileAppend`(`FS_APPEND`/`FS_APPEND_SYNC`).
- **SP is not trap-based for FS** — SP's game module is statically linked
  (per CLAUDE.md module targets) and receives FS entry points as **function
  pointers in a struct**, not syscalls: `game_import_t` has
  `FS_FOpenFile/FS_Read/FS_FCloseFile/FS_ReadFile/FS_FreeFile` members
  (`code/game/g_public.h:196-201`), same shape for the statically-linked UI
  (`code/ui/ui_public.h:36-42`). `GetGameAPI` copies the subset the game
  module actually needs into its own `gameinfo_import_t`
  (`code/game/g_main.cpp:875,907-909`: `FS_FOpenFile`, `FS_Read`,
  `FS_FCloseFile` only — SP's game module doesn't even get
  `FS_ReadFile`/`FS_FreeFile` forwarded to `GI_Init`, though the import
  struct has slots for them). SP's statically-linked cgame gets its
  `CG_FS_FOPENFILE`-style dispatch inline in `code/client/cl_cgame.cpp:462`
  (same case-block shape as MP, just compiled into the exe instead of
  crossing a VM boundary — no marshaling difference to model since it's a
  direct call either way in SP).
- **`fileHandle_t` crossing semantics**: because it's a bare int index into
  the process-global `fsh[]` array, it is safe to pass across the MP VM
  boundary verbatim (both sides see the same engine-owned table — the QVM
  modules never see `fsh` itself, only the integer). For the Rust port this
  means the ABI seam type is just `i32`/`c_int`; the actual `FileHandle ->
  fsh` lookup stays entirely inside the engine's FS module (porting-rules
  §D: unsafe/layout concerns don't apply here since no struct crosses, only
  a plain int).

---

## 6. MP/SP diffs (consolidated)

| Aspect | MP | SP |
|---|---|---|
| Build-canonical pair | `files_common.cpp`+`files_pc.cpp` (jk2mp.vcproj:1466,1497; WinDed.vcproj:219,222; unix makefile `ded` target) | `files_common.cpp`+`files_pc.cpp` (starwars.vcproj:1000,1027) — same pairing |
| Dead variant | `files.cpp` (q3static-only) + `files_console.cpp` (Xbox) | `files_console.cpp` (Xbox only, x_exe.vcproj:264,267) — no `files.cpp` exists in SP tree |
| `MAX_FILE_HANDLES` | 64 (PC, `qcommon.h:507-511`); 16 only under `_XBOX` | 16 unconditional (`files.h:58`) |
| `fs_homepath` | Present, cvar (`files_pc.cpp:2500`) | **Absent** — no homepath cvar or step in `FS_Startup`; all writes/reads relative to `fs_basepath` |
| `fs_basegame` (mod-on-mod) | Present (`files_pc.cpp:2495,2519-2530`) | Absent |
| `fs_dirbeforepak` | Present, gates dir-vs-pak splice order (`files_pc.cpp:2273-2284`) | Absent — packs always prepended to global head (`code/qcommon/files_pc.cpp:1531-1536`) |
| Pure-server (`fs_numServerPaks`, `FS_PakIsPure`, `FS_ReorderPurePaks`, `FS_PureServerSetLoadedPaks/ReferencedPaks`, `FS_ComparePaks`) | Live | **Fully absent as live code.** The Q3 lineage's pure-check call sites are present but **commented out** in SP: `FS_PakIsPure` calls (`code/qcommon/files_pc.cpp:385,747`), `fs_numServerPaks` guard (L433), and the whole checksum-match block (L1396-1402) are all inside `/* */` — dead, not deleted. `fs_serverPaks`/`fs_numServerPaks` declarations themselves are also commented (L19-20). |
| `pack_t.pure_checksum`, `.pakBasename`, `.pakGamename`, `.referenced` | Present (`files.h:42-56`) | Absent (`code/qcommon/files.h:33-43`) — SP `pack_t` has only `pakFilename/handle/checksum/numfiles/hashSize/hashTable/buildBuffer` |
| `fileHandleData_t.streamed` field | Present | Absent |
| `fs_checksumFeed`/`fs_fakeChkSum` | Present, live (keyed pak checksums, §2) | Absent entirely — no keyed checksum concept |
| `FS_Restart` signature | `FS_Restart(int checksumFeed)` (`files_pc.cpp:2988`) | `FS_Restart(void)` (`code/qcommon/files_pc.cpp:1667`) — no feed param |
| `FS_ConditionalRestart` | Present (`files_pc.cpp:3048`) | **Absent** — no SP call site found; nothing needs it without pure-server reconnect flow |
| `lastValidBase`/`lastValidGame` fallback in `FS_Restart` | Present (`files_pc.cpp:3011-3021`) | Not present in SP's much simpler `FS_Restart` (just fatals on missing `default.cfg`) |
| Demo-pak checksum guard | `FS_SetRestrictions` XORs with `0x02261994u` (`files_pc.cpp:2632`) | Same mechanism, different XOR constant `0x10228436u` and message text ("Corrupted pk3" vs "Corrupted pak0.pk3") — `code/qcommon/files_pc.cpp:1653-1654` |
| `NUM_ID_PAKS`/`FS_idPak` (never-autodownload id paks) | Present (`qcommon.h:505`, `files_pc.cpp:2301-2313`) | Absent — no autodownload concept in SP at all |
| Base data pak naming | Checked via `"%s/assets%d"` (`files_pc.cpp:2305`) despite comments elsewhere still saying "pak0.pk3" (stale Q3-lineage comment text, e.g. L2253,2633) — actual JKA data files are `assetsN.pk3`, not `pakN.pk3` | Same `assetsN.pk3` naming (SP ships `assets0-2.pk3`); no `FS_idPak`-equivalent needed since no autodownload |
| Module FS seam | VM syscalls (`G_FS_*`/`CG_FS_*`/`UI_FS_*`) — game/cgame/ui all separate QVMs | Function-pointer struct injection (`game_import_t`/`ui_import_t`); cgame+ui statically linked, no VM boundary at all |
| Trap-vs-static caveat | 3 separate loadable modules per CLAUDE.md | Only `jagame` is a loadable module; cgame/ui are compiled into the exe (per CLAUDE.md module targets) |

---

## 7. TU-harness candidates (DEC-09)

- **Pak-parsing/checksum standalone feasibility**: **high.** `FS_LoadZipFile`
  (`files_pc.cpp:1423-1522`) has no dependency on `fs_searchpaths` or any
  other FS global except the read-only `fs_checksumFeed` (for the keyed
  checksum) — it's a pure function of `(zipfile path, basename,
  checksumFeed) -> pack_t`. A golden-file harness can feed it a small
  in-repo fixture `.pk3` (or a synthesized zip via the Rust `zip` crate) and
  assert `checksum`/`pure_checksum` against captured oracle output. The
  keyed-checksum math itself (`Com_BlockChecksum`/`Com_BlockChecksumKey`,
  referenced but defined in `qcommon/common.cpp`, not `files_*`) is an
  independent, easily-fixtured pure function over `int[]` — good
  TU-harness unit on its own, decoupled from zip parsing entirely.
- **Search-order goldens**: **high**, but needs a small synthetic directory
  tree (not the real oracle assets) since `FS_AddGameDirectory`'s ordering
  logic (`files_pc.cpp:2211-2294`) depends only on `Sys_ListFiles` output +
  `fs_dirbeforepak`, both mockable. A golden test matrix over
  `{fs_dirbeforepak: 0/1} x {basepath, homepath, basegame, moddir present:
  2^4}` exercising `FS_Startup`'s ordering (§1b) would pin the priority
  rules mechanically rather than by re-reading the C each time. SP's
  matrix is much smaller (no `fs_dirbeforepak`/`fs_homepath`/`fs_basegame`
  axes) — worth a **separate, smaller** SP golden rather than reusing MP's.
- **Pure-server matching** (`FS_PakIsPure`, `FS_ReorderPurePaks`,
  `FS_ComparePaks`): mockable purely over a list of `(checksum, name)`
  pairs plus a server-sent list — no filesystem I/O needed once pak
  objects exist. Good MP-only TU-harness target; **not applicable to SP**
  (dead code there, §6) so don't build a shared harness that assumes both
  sides need it.
- **Handle table / `FS_FOpenFileRead` walk**: harder to isolate cleanly
  since it's entangled with `unzReOpen`/real file I/O and the
  Windows-only `fs_copyfiles==2` branch (`files_pc.cpp:897-967`,
  `#ifndef __linux__`) — that branch should probably be scoped out of
  parity requirements for a Linux/macOS-hosted golden harness rather than
  faithfully ported, pending a design-doc decision (flag for design
  session, not a fork call here).

---

## Design forks

1. **Handle table shape.** Raven's `fsh[]` is a fixed-size array
   (`64` MP-PC / `16` SP) of tagged unions, scanned linearly for a free
   slot, with `0` reserved as "invalid." A Rust port could keep a flat
   `Vec<Option<FileHandleData>>`/slab with an equivalent "index 0 unused"
   convention to keep ABI-crossing `FileHandle` trivially `i32`-compatible,
   or drop the historical index-0 sentinel and use `Option<NonZeroU32>` at
   the safe-Rust boundary while still exposing `0 = None` at the seam.
   Needs a decision before porting `fsh`/`FS_HandleForFile`.
2. **Zip crate mapping vs raw `unzip.h` semantics.** The vendored
   `unzip.cpp` exposes a streaming, seek-by-position API
   (`unzGetCurrentFileInfoPosition`/`unzSetCurrentFileInfoPosition`,
   `files_pc.cpp:832-838`) that the Rust `zip` crate doesn't mirror
   1:1 (it indexes by name/index, not raw offset). Need a decision on
   whether the Rust `pack_t` equivalent pre-resolves an index (via the
   `zip` crate's `ZipArchive::by_index`) at hash-table build time instead
   of storing a raw byte offset — behaviorally equivalent for lookups but
   changes what "reopen at position" means for `uniqueFILE` handles
   (§3, `unzReOpen`/`unzOpenCurrentFile`).
3. **Case-sensitivity policy.** `FS_FilenameCompare`/`FS_PathCmp`
   (`files_common.cpp:345-372`, `files_pc.cpp:2041-2071`) both fold ASCII
   case for *logical* qpath comparisons and pak hash-table lookups, but the
   actual `fopen`/`Sys_ListFiles` calls hit the real filesystem
   case-sensitively-or-not depending on OS (`unix/unix_shared.cpp:159-234`
   uses `Q_stricmp` only for the *extension* suffix match, not the whole
   name — directory listing itself is whatever `readdir` returns, and
   `fopen` on Linux is fully case-sensitive while macOS default
   (APFS/HFS+) is case-insensitive-but-preserving). A Rust port targeting
   Linux+macOS+Windows needs an explicit decision: normalize all qpaths to
   lowercase before touching the real FS (matching pak-lookup semantics
   uniformly), or accept host-OS-dependent behavior like the oracle does.
4. **Homepath introduction for SP.** Raven's SP genuinely has no
   `fs_homepath` (§6) — all reads/writes (saves, configs) go through
   `fs_basepath`/`fs_gamedirvar`. A modernized Rust SP port could choose to
   introduce a homepath-style separation (common on Linux ports needing a
   writable dir separate from a read-only install), but doing so is a
   **behavioral addition beyond the oracle**, which porting-rules §A.2
   ("No speculative behavior... port it faithfully first") argues against
   for the initial port. Flag explicitly as a "faithful now, revisit
   later" decision rather than silently adding it.
5. **`fs_dirbeforepak`/pure-server fields on SP's `pack_t`.** Since SP's
   `pack_t` is a strict subset of MP's (§6), and both crates presumably
   want to share a `Pack`-shaped type in `qshared` per the workspace tiers
   doc — decide whether SP gets the full MP-shaped struct with unused
   fields (simpler generic code, wasted bytes) or a genuinely narrower SP
   type (faithful-to-oracle size, more enum/cfg-gated code paths). Given
   porting-rules §B.12 ABI-crossing fidelity only binds fields that
   actually cross a boundary, and `pack_t` is engine-internal (never
   crosses to game/cgame/ui), the narrower per-mode struct is likely
   preferable — but this is a call for the design session, not decided
   here.
6. **`fileHandle_t`/`fsMode_t` enum-vs-alias fidelity** (CLAUDE.md
   convention, already decided by that rule, noted here for the design
   doc's benefit): `fileHandle_t` is `typedef int`
   (`codemp/game/q_shared.h:362`) → plain `type FileHandle = c_int` alias,
   no enum. `fsMode_t` is a true named C enum
   (`FS_READ/FS_WRITE/FS_APPEND/FS_APPEND_SYNC`, `q_shared.h:1685-1690`) →
   must become a real `#[repr(i32)] enum FsMode`, not flattened — exactly
   the `spectatorState_t`/`alertEvent*` pitfall CLAUDE.md warns about, so
   call it out explicitly when `fsMode_t` is ported.
