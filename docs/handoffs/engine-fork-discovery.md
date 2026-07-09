# Engine pre-port fork discovery (2026-07-09) — RULINGS PENDING

The engine equivalent of `jampgame-fork-discovery.md`: the design forks that
must be user-ruled BEFORE the mega-pass transcription window, so porters are
blind executors of settled decisions (see plan §"Port-process discipline" and
`docs/GOAL-engine.md`). Derived from the corrected dependency walk
(`tools/closure-prototype/out/engine/engine-port-order.{json,tsv,md}`, 2,481
fns / 87,728 LOC / 5,081 edges). Each fork lists a RECOMMENDED position;
**none are settled until ruled inline.**

## Fork classes (blast-radius order)

1. **Error recovery: `Com_Error` is a `longjmp`** (`abortframe` setjmp in
   `Com_Frame`/`Com_Init`; ERR_DROP unwinds mid-frame from anywhere — the
   110-fn core knot exists *because* error paths call back into everything).
   RECOMMENDED: Rust panic + `catch_unwind` at exactly the Raven setjmp
   sites; payload carries the error level/message; `com_error_recover` (the
   existing stub) becomes the landing pad. No `Result` threading — it would
   rewrite every signature in the engine and break transcription-first.
   **RULING: PENDING**

2. **Global state placement** (~680 file-scope globals: `sv` 665KB, `svs`,
   `cvar_indexes`/hash, `fs_*` pak state, `cm` clipmap, `msgHuff` 102KB,
   `com_*` cvar handles, botlib's `aasworld`/`botlibglobals`, tr_ state).
   RECOMMENDED: the jampgame fork-1 pattern — everything becomes fields on
   the owning subsystem struct under the existing `Engine` aggregate
   (`Engine { common, sv, cm, fs, net, bot, g2, ... }`), grouped by owning
   .c file; engine cvar *handles* in one `EngineCvars` sub-struct per
   subsystem. No `static mut` anywhere (const tables stay `const`).
   **RULING: PENDING**

3. **Function-scope statics (119: qcommon 48, ghoul2 40 — the RagDoll
   solver, `CM_LoadMap_Actual::last_checksum`, botlib `AAS_ContinueInit*`
   frame counters).** RECOMMENDED: bless the jampgame fork-5 three-kind rule
   unchanged — const tables → `const`; rotating scratch/return buffers →
   owned return values; genuine cross-frame state → host-struct fields
   (fork 2). **RULING: PENDING**

4. **Memory allocators: Zone (`Z_Malloc` tags) + Hunk (two-sided marks,
   temp/perm) + `Hunk_AllocateTempMemory`.** Allocation ORDER and reuse are
   parity-visible wherever pointers/indices leak into state the referee
   diffs. RECOMMENDED: port allocator logic faithfully as owned arenas
   (`Vec<u8>`-backed, same mark/free-list semantics, deterministic layout);
   no Rust global allocator substitution on parity paths; idiomatization
   deferred to the safe-state migration. **RULING: PENDING**

5. **Internal dispatch tables** (`botlib_export_t`/`botlib_import_t`,
   `refexport_t` (1 live arm under DEDICATED), ICARUS `interface_export_t`
   ~40 fns, `ucmds[]`-style command tables). These never cross the module
   ABI; grep found no address-comparison of their members (unlike the
   jampgame entity handlers). RECOMMENDED: plain Rust structs of `fn` items
   populated at the same init sites (1:1 shape, zero indirection cost, keeps
   the 261 ref-edges meaningful); command tables as `&[(&str, fn(...))]`
   consts. Fn-ID enums NOT needed absent address compares. **RULING: PENDING**

6. **VM subsystem stance** (`vm.cpp`/`vm_interpreted.cpp`/`vm_x86.cpp`; plan
   §5.4). RECOMMENDED: all three port; interpreter is portable logic;
   `vm_x86` ports as data-faithful emitter (executes only on x86 hosts, same
   as C); runtime path for our module stays native-dylib; interface-crate
   arg slots are `intptr_t`-width. **RULING: PENDING**

7. **C++-track design docs (§F) — which subsystems get one, before the
   window.** GP2 is already done (the pilot). RECOMMENDED docs: **icarus**
   (253 fns: CTaskManager/CSequencer/CBlockStream tree), **RMG** (113:
   CRMManager/instance hierarchy — closed → enums), **ghoul2 + renderer
   class internals** (CGhoul2Info_v arena + CBoneCache), **CNavigator**
   (server/NPCNav, 81 fns — engine-side twin of the game's nav API),
   **CROFFSystem** (RoffSystem.cpp). Everything else is C-track packets.
   **RULING: PENDING (list + per-doc scope)**

8. **The platform trait** (the 26 `Sys_*`/`NET_*` externals + the main loop
   from `null/win_main.cpp`, excluded from the port set). RECOMMENDED: one
   `PlatformHost` trait in the interface crate (clock, console I/O, UDP,
   file listing, dylib loading — the module loader already exists);
   deterministic test impl for the referee (fixed clock, scripted packets),
   std impl for the real binary. Unix path semantics (the referee platform),
   not Win32. **RULING: PENDING**

9. **Filesystem semantics** (`files_common.cpp`/`files_pc.cpp`): search-path
   order, pure-server pak checksum lists, `fs_homepath`/`fs_basepath`,
   macOS case-insensitivity vs Raven's case assumptions. Parity-visible
   through configstrings (`sv_paks`) and download lists. RECOMMENDED:
   faithful port of ordering/checksum logic over the platform trait's
   directory enumeration, with enumeration order pinned (sorted) and a
   golden fixture on the retail-assets pak list. **RULING: PENDING**

10. **Console/print routing** (`Com_Printf` → console/logfile — feeds the
    referee's syscall-stream digest via `G_PRINT` echo and `Sys_Print`).
    RECOMMENDED: route through the platform trait; byte-identical
    formatting (`va`/`Com_sprintf` already ported in qshared); no timestamps
    unless Raven prints them. **RULING: PENDING**

## The type rosetta (agent packet reference)

`tools/closure-prototype/out/engine/type-rosetta.tsv` — generated by
`tools/closure-prototype/typemap.py` from the house-style `Source:` cites:
**2,702 ported items, 2,002 distinct Raven type names** → Rust name, kind,
crate, and file path. Every porting packet includes (or references) this
index. Agent rules it enforces:

- **All ABI/layout types already exist** (type port complete). A porter
  NEVER declares a struct/enum/typedef — it imports from the listed path.
- **A name missing from the rosetta is an escalation, not a stub** — the
  no-stub discipline (plan §"Port-process discipline") applies to types
  exactly as to functions. The finisher triages misses (usually a naming
  variant; the tool is regenerated after any legitimate addition).
- Regenerate with `.venv/bin/python typemap.py` after any type lands or
  moves; the TSV is generated, never hand-edited.

## Not forks (already settled elsewhere)

- Transcription-first / no safety refactoring during the port (plan §1).
- Vendored zlib/png via Rust crates; platform files excluded (plan §1).
- FINAL_BUILD undefined; WinDed vcproj Release macro set (plan appendix).
- One engine in the repo; interface crate first (referee plan).
