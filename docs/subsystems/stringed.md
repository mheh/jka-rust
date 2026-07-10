---
files:
  - path: crates/mp/engine/qcommon/src/stringed/mod.rs
    crate: mp_engine_qcommon
    mode: mp
    class: "(module)"
    summary: "Module doc + re-exports; SE_* text-equate consts (iSE_VERSION=1, iSE_MAX_FILENAME_LENGTH=MAX_QPATH, sSE_STRINGS_DIR=\"strings\", the sSE_KEYWORD_* parse tokens, sSE_*_FILE_EXTENSION, sSE_EXPORT_SAME=\"#same\", sSE_DEBUGSTR_PREFIX/SUFFIX). Source cites: stringed_ingame.h:10-34."
  - path: crates/mp/engine/qcommon/src/stringed/entry.rs
    crate: mp_engine_qcommon
    mode: mp
    class: SeEntry
    summary: "Raven `SE_Entry_s` (renamed per SE-D2 — internal, std::string members, not ABI): { string m_str_string; string m_str_debug; i32 m_i_flags }. One localized-string record. Owned `String`/`i32` (§C9); ctor zeroes m_iFlags. Source: stringed_ingame.cpp:48-59."
  - path: crates/mp/engine/qcommon/src/stringed/package.rs
    crate: mp_engine_qcommon
    mode: mp
    class: StringEdPackage
    summary: "Raven `CStringEdPackage` (renamed per SE-D2/RULING 40 C-prefix drop): the 20 methods + the parse-only scratch fields + the entry store (`BTreeMap<String,SeEntry>`) + flag names (`Vec<String>`) + flag masks (`BTreeMap<String,i32>`) + m_bLoadDebug (SE-D1). This struct is the `Engine.common.stringed` field (SE-D1(1)/SE-D6, no singleton). Parse/lookup/flag methods take `&self`/`&mut self`; SE_GetString's debug arm reads se_debug via `&mut impl EngineHost` (SE-D3). Filename_*/InsideQuotes/ConvertCRLiterals_Read/Leetify static scratch buffers become owned returns (SE-D1(3), RULING 3 three-kind)."
  - path: crates/mp/engine/qcommon/src/stringed/api.rs
    crate: mp_engine_qcommon
    mode: mp
    class: "(se_* load/enumeration free-fn API)"
    summary: "The load/enumeration free-function API — idiomatic snake_case per RULING 40/SE-D7 (internal Rust→Rust, `&mut StringEdPackage` first param, not link/syscall targets): se_init/se_shut_down/se_new_language, se_load/se_load_language/se_check_for_language_updates, se_get_num_languages/se_get_language_name/se_get_language_dir; the file-static helpers leetify/cope_with_dumb_string_data/se_load_actual/se_get_found_file; the gvLanguagesAvailable cache. The arity-overloaded LOOKUP getters (Raven `SE_GetString`/`SE_GetFlags` pairs + `SE_GetNumFlags`/`GetFlagName`/`GetFlagMask`) are NOT here — they are `StringEdPackage` seam methods (get_string/get_string2/get_flags/get_flags2/get_num_flags/get_flag_name/get_flag_mask on package.rs, SE-D7/RULING 57). Each free fn threads `&mut engine.common.stringed` + `&mut impl EngineHost`. Language selection/enumeration/registration served by the 19-method EngineHost (SE-D3, RULING 55: cvar_register/cvar_string/cvar_take_modified/fs_list_files) — fully unblocked, no open host gap."
  - path: crates/mp/engine/qcommon/src/stringed/interface.rs
    crate: mp_engine_qcommon
    mode: mp
    class: "(SE_* interface — stringed_interface.cpp)"
    summary: "The engine-side interface TU (WinDed link set, SE-D4): SE_LoadFileData (FS_ReadFile path), SE_FreeFileDataAfterLoad (FS_FreeFile), SE_BuildFileList + SE_R_ListFiles (recursive FS_ListFiles scan → ';'-delimited string). Only the `#ifndef _STRINGED` in-engine branches port; the `_STRINGED` editor-tool branches are §20 drops (SE-V1). Directory scan served by `EngineHost::fs_list_files` (SE-D3, RULING 55)."
divergences:
  - id: SE-V1
    site: "oracle/codemp/qcommon/stringed_interface.cpp:49-84,201-211"
    note: "The `_STRINGED` branches (raw fopen/filesize/malloc in SE_LoadFileData, `BuildFileList`+extern `strResult` in SE_BuildFileList) are the standalone StringEd-editor tool build, NOT the in-engine build. `_STRINGED` is undefined in the WinDed/engine TU, so only the `#ifndef _STRINGED` paths (FS_ReadFile, SE_R_ListFiles) compile. Port the in-engine branches only; the editor branches are §20 zero-caller drops."
  - id: SE-V2
    site: "oracle/codemp/qcommon/stringed_ingame.cpp:109,111,113"
    note: "`CStringEdPackage::GetNumStrings()`, `SetReference(int,LPCSTR)`, `GetCurrentFileName()` are DECLARED in the class body but never defined and never called anywhere in the TU. §20 zero-caller drops — not ported, module-doc note only."
  - id: SE-V3
    site: "oracle/codemp/qcommon/stringed_ingame.cpp:818,1033,1051,1127,1140"
    note: "`__ASSERT(0)`/`__ASSERT(strlen<...)` in SetString-miss, SE_GetFlags(single) miss, SE_GetFlagName OOR, SE_GetLanguageName/Dir OOR. `__ASSERT` = `assert` (stringed_ingame.cpp:46); under the WinDed Release macro set (`-DNDEBUG`, plan appendix) `assert` is a no-op, so the faithful behavior is the fall-through return (0 / \"\"). Reproduce the fall-through return; the assert never fires in the ported build (do not panic)."
  - id: SE-V4
    site: "oracle/codemp/qcommon/stringed_ingame.cpp:1046,1122,1135"
    note: "`iFlagIndex < m_vstrFlagNames.size()` / `iLangIndex < ...size()` compare a signed `int` against `size_t`; C promotes the int to unsigned, so a NEGATIVE index wraps to a huge value and fails the bound (returns \"\"). Preserve the observable result: treat negative indices as out-of-range → \"\" (an explicit `idx < 0 || idx >= len` guard reproduces it; do not accidentally allow negative)."
  - id: SE-V5
    site: "oracle/codemp/qcommon/stringed_ingame.cpp:483-574,681-713"
    note: "CopeWithDumbStringData Z_Malloc's a `strlen*3` scratch buffer (for the 0x85 ellipsis 1→3 expand via memmove) and Z_Free's it in ParseLine. Manual alloc → owned `String`/`Vec<u8>` (§C9); the hi-char substitutions and the ellipsis expansion are reproduced value-for-value, layout-free. The `*3` sizing is a C buffer-safety detail, not observable output."
  - id: SE-V6
    site: "oracle/codemp/qcommon/stringed_ingame.h:71-104"
    note: "The `Language_Is{Russian,Polish,Korean,Taiwanese,Japanese,Chinese,Thai}` inline header helpers have zero callers in the codemp `.c/.cpp` tree (only the renderer/font `Language_IsAsian`, a separate trap, exists) — and the renderer is not in the DEDICATED/WinDed link set. §20 dead surface under this build: port the equates but expect no live caller; retain for future client/renderer waves. Each reads se_language->string (now `EngineHost::cvar_string`, SE-D3)."
---

# StringEd localization (CStringEdPackage) — MP engine (§F idiomatic reimplementation)
Status: REVIEWED     Supersedes: none
Decision prefix: SE     Ledger deps: DEC-09 (oracle-differential parity), DEC-04 (per-mode)

## Standing context
Links only — never restate:
- `docs/porting-rules.md` — §F (C++-track idiomatic reimplementation, rules
  17-21), §B (state ownership, no globals), §C (C→Rust idiom: out-params→returns,
  manual alloc→ownership), §19/§20 (UB / dead-surface notes), comment/source-cite
  rules.
- `docs/doc-standards.md` — this template + gates; rule 6 (C++-track roster +
  divergences frontmatter, consumed by `port-cpp-subsystem`'s `designPath`).
- `docs/GOAL-engine.md` — pure-Rust dedicated `openjkded`; total scope, no stubs.
- `docs/plans/2026-07-08-mp-engine-build-out.md` — port order/waves; Stage 0 (the
  game-host interface crate that defines `EngineHost`); appendix (WinDed Release
  macro set: `-DNDEBUG -DDEDICATED -DBOTLIB`, `_STRINGED`/`FINAL_BUILD` undefined).
- `docs/handoffs/engine-fork-discovery.md` — settled forks + §F doc rulings:
  **RULING 50** (this subsystem's seed: `Engine.common` sub-struct field,
  store-owns-Strings/lookups-borrow-&str, Filename_* → ruling-3 three-kind,
  BTreeMap for Raven's sorted map order), **RULING 3** (function-scope statics
  three-kind rule: const tables → `const`, rotating scratch/return buffers →
  owned returns, cross-frame state → host-struct fields), **RULING 40** (§F
  naming: drop the bare hungarian `C` prefix; ABI-frozen types keep exact Raven
  names; internal = idiomatic naming), **RULING 31/33/36** (`EngineHost` is built),
  **RULING 55** (2026-07-09, the host-closing ruling: `EngineHost` EXTENDED to 19
  methods and BUILT — commit b2855df2 — closing SE-Q1/SE-Q2, and the
  construction-story correction, SE-D6), **RULING 57** (2026-07-09, this
  subsystem's closing ruling: the arity-overloaded `SE_GetString`/`SE_GetFlags`
  lookup API is idiomatic `StringEdPackage` methods, not re-exported C names —
  closing SE-Q3, SE-D7), **RULING 32** (MockHost drives the
  goldens), **RULING 44** (Win32 `long` in any binary/file format = 4 bytes — see
  SE-D5 applicability), **RULING 2** (`Engine`/`Common` sub-structs, no
  `static mut`).
- `docs/workspace-architecture.md` — crate graph (qcommon tier; `core` defines
  `Engine`, holds `Engine.common: Common`).
- `docs/architecture/state-ownership.md` — the Engine **construction convention**
  this doc's new `common.stringed` field obeys (SE-D6): `Engine::new()`
  `alloc_zeroed`s the whole aggregate, then `.write()`s every field that is **not**
  all-zero-valid before `assume_init` (STATE-D5, LIFE-Q9, LIFE-D4b; the
  `//TODO: Port ZeroValid` note at `core/src/engine.rs:41-48`). There is **no**
  `Common::default()`.
- `docs/abi-traps.md` — row **52** `trap_SP_GetStringTextString` (the inbound
  game-module syscall this subsystem serves).
- Exemplar: GP2 (`crates/mp/engine/qcommon/src/gp2/`, `tools/gp2-oracle/`); the
  closest §F sibling roster/verification shape is `docs/subsystems/roff.md`.

## Scope & non-goals
Decides the Rust shape, ownership, seam, and verification of the **StringEd
localization package** as the WinDed/engine link set uses it — **both** TUs the
project links:
- `oracle/codemp/qcommon/stringed_ingame.cpp` (+ `.h`): `CStringEdPackage` (its
  20 methods), `SE_Entry_s`, the `SE_*` public C API, the `Leetify` /
  `CopeWithDumbStringData` / `SE_Load_Actual` / `SE_GetFoundFile` file-static
  helpers, and the `gvLanguagesAvailable` language cache.
- `oracle/codemp/qcommon/stringed_interface.cpp` (+ `.h`): `SE_LoadFileData`,
  `SE_FreeFileDataAfterLoad`, `SE_BuildFileList`, `SE_R_ListFiles`.

**Link-set verification (SE-D4).** `oracle/codemp/WinDed.vcproj:258-267` lists all
four StringEd files — `qcommon\stringed_ingame.cpp`, `stringed_ingame.h`,
`stringed_interface.cpp`, `stringed_interface.h`. So **both** `.cpp` TUs are in
scope; `stringed_interface.cpp` IS linked (confirmed, per the task's verify-first
instruction).

Non-goals (punted, with pointers):
- **The `EngineHost` method roster** (which FS/print/cvar methods the trait
  exposes) is owned by the Stage-0 interface-crate design (RULING 11/31/33/36/55),
  not here. As of **RULING 55** the trait is **built and complete for StringEd's
  needs** (19 methods, commit b2855df2): every service StringEd consumes has a
  method (SE-D3). This doc names those services and takes `&mut impl EngineHost`; it
  invents no trait method and — with SE-Q1/SE-Q2 now closed by RULING 55 — carries
  **no** open host gap.
- **The SP StringEd** (`oracle/code/qcommon/stringed_*.{h,cpp}`) is the SP-engine
  twin; per DEC-04 it is a separate MP-then-SP-diff exercise, not this doc.
- **`trajectory_t`/`sharedEntity_t`/VM marshalling layout** of the inbound trap
  is owned by the server / abi-rosetta docs; this doc names only the copy the trap
  arm performs.

## Raven ground truth
CITE OR OMIT. Paths are `oracle/`-relative.

**One global package instance.** `CStringEdPackage TheStringPackage;`
(`codemp/qcommon/stringed_ingame.cpp:124`). It holds: the parse-only scratch state
(`m_bEndMarkerFound_ParseOnly`, `m_strCurrentEntryRef_ParseOnly`,
`m_strCurrentEntryEnglish_ParseOnly`, `m_strCurrentFileRef_ParseOnly`,
`m_strLoadingLanguage_ParseOnly`, `m_bLoadingEnglish_ParseOnly`, `:68-73`); the
entry store `mapStringEntries_t m_StringEntries` — a `map<string,SE_Entry_t>`
(`:62,87`); `SE_BOOL m_bLoadDebug` (`:88`); and the flag tables
`vector<string> m_vstrFlagNames` + `map<string,int> m_mapFlagMasks` (`:92-93`).
Ctor/dtor both call `Clear(SE_FALSE)` (`:79,84`).

**`SE_Entry_s`** (`:48-59`): `string m_strString` (the resolved localized text),
`string m_strDebug` (english/"#same" debug text, prefixed `[`…`]`), `int m_iFlags`
(bitmask). Ctor zeroes `m_iFlags`. Members are `std::string` — **not ABI**, so it
is renamed `SeEntry` per SE-D2.

**Other file-scope globals (must appear in the state table):** the three cvar
pointers `se_language`, `se_debug`, `sp_leet` (`:41-43`; registered in `SE_Init`,
`:1169-1171`); `int giFilesFound` (interface.cpp:131, the list-scan counter);
`vector<string> gvLanguagesAvailable` (`:1071`, the cached language list).

**Clear** (`:127-152`): clears `m_StringEntries`; **only when `!bChangingLanguages`**
also clears the flag tables — the comment (`:133-141`) explains flags stay defined
once seen (so cached game-side flag masks stay valid across a language reload);
only the dtor kills them. Resets the parse-only end-marker + ref scratch.

**Filename helpers (SE-D1(3)).** Each uses its **own** `static char
sString[iSE_MAX_FILENAME_LENGTH]`:
- `Filename_PathOnly` (`:162-175`): strcpy, find last `\` or `/`, truncate → dir.
- `Filename_WithoutExt` (`:184-203`): strcpy, find last `.`, truncate only if it
  is past the last slash (guards a path with no extension).
- `Filename_WithoutPath` (`:211-227`): scan for the last slash, strcpy the tail.
- `ExtractLanguageFromPath` (`:230-233`): `Filename_WithoutPath(Filename_PathOnly(f))`.
**Aliasing check (SE-D1(3)):** the two call chains that nest these —
`ExtractLanguageFromPath` (PathOnly→WithoutPath) and `SetupNewFileParse`
(WithoutExt→WithoutPath, then strcpy to a local, `:240`) — nest **distinct**
helpers, each with its own static, and the outer reads the inner's result into a
fresh buffer before the next call. **No caller ever holds two results of the same
helper at once**, so owned-`String` returns are faithful (no observable reuse).

**SetupNewFileParse** (`:236-247`): file ref = `Filename_WithoutPath(WithoutExt(f))`
uppercased (`Q_strupr`) → e.g. `"OBJECTIVES"`; loading-language =
`ExtractLanguageFromPath(f)`; `m_bLoadingEnglish_ParseOnly` = (lang == "english");
`m_bLoadDebug = bLoadDebug`.

**Line reader.** `ReadLine` (`:344-384`): splits on `\n`, copies one line, skips
`\r\n` runs, right-trims whitespace, then `REMKill`. `REMKill` (`:292-340`): kills
a `//`-comment tail **unless inside quotes** (parity of double-quote count),
right-trims. `CheckLineForKeyword` (`:254-270`): case-insensitive prefix match
(`Q_stricmpn`); on match advances the ptr past the keyword + whitespace.
`InsideQuotes` (`:388-436`, static `string str`): strip leading ws + opening quote,
strip trailing ws + closing quote. `ConvertCRLiterals_Read` (`:275-287`, static
`string str`): rewrite the 2-byte `\n` literal → 1-byte newline.

**Parse dispatch.** `ParseLine` (`:578-727`) keyword-matches, in order:
`VERSION` (must equal `iSE_VERSION`=1 else error), `CONFIG`/`FILENOTES`/`NOTES`
(absorbed, ignored), `REFERENCE` → `AddEntry(InsideQuotes(...))`, `FLAGS` →
tokenize on ` \t`, uppercase each, `AddFlagReference`, `ENDMARKER` → set the
end-marker flag, `LANG_` (prefix) → parse language word + `InsideQuotes` sentence,
run `CopeWithDumbStringData`, then `SetString`; unknown keyword → error string.
Returns `NULL` for ok, else a `va()`-formatted error message.

**Entry mutation.** `AddEntry` (`:738-750`): key = `va("%s_%s", fileRef, localRef)`;
insert an empty `SE_Entry_t` if absent (never overwrite — comment `:740-742`: .STE
override files carry no flags, must not wipe parsed flags), set current ref.
`SetString` (`:775-820`): find key; if english/debug-english, store
`Leetify(psNewString)` into `m_strString`, and if `m_bLoadDebug` build `m_strDebug`
= `[` + text + `]`, cache the english for later `#same`; else (foreign) if text ==
`"#same"` (`sSE_EXPORT_SAME`) copy the cached english, else store foreign text.
Miss → `__ASSERT(0)` (SE-V3). `Leetify` (`:752-772`, static `string`): if
`sp_leet->integer == 42`, char-substitute (o→0, l→1, …).

**Flags.** `AddFlagReference` (`:454-474`): if the flag name is new, push to
`m_vstrFlagNames` and set `m_mapFlagMasks[name] = 1 << (size-1)` — **so the bit is
the first-seen encounter index** (parity-critical, driven by the `Vec` push order,
independent of the map); then OR the mask into the current entry's `m_iFlags`.
`GetFlagMask` (`:441-451`): map find → mask, else 0.

**`CopeWithDumbStringData`** (`:483-574`, file-static): `Z_Malloc(strlen*3)`,
`Q_strncpyz`, then — for ENGLISH/FRENCH/GERMAN/ITALIAN/SPANISH/POLISH/RUSSIAN
only — substitute hi-chars (0x92→`'`, 0x93/0x94→`"`, 0x0B→`.`, 0x85→`...` via
memmove-expand, 0x91→`'`, 0x96/0x97→`-`), fix `"?."`→`"? "`, tabs→space. Returned
buffer is `Z_Free`d by the caller (`:713`). → owned `String`/`Vec` (SE-V5).

**Public API — the load path.** `SE_Init` (`:1156-1196`): `Clear(SE_FALSE)`;
`Cvar_Get` se_language ("english", `CVAR_ARCHIVE|CVAR_NORESTART`), se_debug ("0"),
sp_leet ("0", `CVAR_ROM`); if `com_buildScript->integer == 2` load every language;
then `SE_LoadLanguage(se_language->string)`, `Com_Error(ERR_DROP,…)` on failure.
`SE_Load` (`:910-966`): if the name has no `/`, prepend `strings/<se_language->string>/`;
`COM_DefaultExtension(".str")`; `SE_Load_Actual(...,SE_FALSE)`; then try the
matching `.ste` override with `SE_Load_Actual(...,SE_TRUE)` (speculative). On error
`Com_Error(ERR_DROP)` if critical else `Com_DPrintf`. `SE_Load_Actual`
(`:828-873`, static): `SE_LoadFileData` → `SetupNewFileParse` → loop
`ReadLine`/`ParseLine` over a 16384-byte line buffer → `SE_FreeFileDataAfterLoad`
→ require `EndMarkerFoundDuringParse` else "Truncated file" error.
`SE_LoadLanguage` (`:1208-1243`): `SE_NewLanguage()` (`Clear(SE_TRUE)`), then
`SE_BuildFileList("strings", results)` and `SE_Load` each file whose
`ExtractLanguageFromPath` matches. `SE_CheckForLanguageUpdates` (`:1250-1261`,
called every `Com_Frame`): if `se_language->modified`, reload the language, clear
`modified`. `SE_GetNumLanguages` (`:1072-1116`): lazy — if `gvLanguagesAvailable`
empty, `SE_BuildFileList` + `SE_GetFoundFile` loop, dedup via a `set`, english
first. `SE_GetLanguageName`/`SE_GetLanguageDir` (`:1120-1142`): index into the
cache (OOR → `__ASSERT`, SE-V3/V4).

**Public API — lookup (the LIVE seam).** `SE_GetString(pkg, ref)` (`:971-978`):
`sprintf("%s_%s")`, uppercase, call the 1-arg form. `SE_GetString(ref)`
(`:981-1007`): copy+uppercase (256-byte buffer, `assert(len<256)`), `find`; on hit
return `Entry.m_strDebug.c_str()` when `se_debug->integer && m_bLoadDebug`, else
`Entry.m_strString.c_str()`; **miss → return `""`** (no active assert). The
returned `const char*` points into the map entry's `std::string` storage, stable
until the next `Clear` (language reload) — **SE-D1(2)**. `SE_GetFlags` (both,
`:1012-1036`), `SE_GetNumFlags` (`:1039-1042`), `SE_GetFlagName` (`:1044-1053`),
`SE_GetFlagMask` (`:1057-1060`) are pure package reads. **Iteration note:**
`m_StringEntries`/`m_mapFlagMasks` are only ever `find()`-ed in this TU (never
iterated), so their `std::map` ordering is not directly output-visible here; SE-D1(4)
nonetheless keeps `BTreeMap` for determinism/faithfulness. The behavior-visible
ordering is the flag `Vec` push order (above).

**Interface TU** (`stringed_interface.cpp`, in-engine `#ifndef _STRINGED` paths).
`SE_LoadFileData` (`:41-104`): `FS_ReadFile(name,&buf)`; `iLen>0` → return buffer +
optional length. `SE_FreeFileDataAfterLoad` (`:109-122`): `FS_FreeFile`.
`SE_BuildFileList` (`:192-212`): reset `giFilesFound`, `SE_R_ListFiles(".str", dir,
results)`, return count. `SE_R_ListFiles` (`:132-184`): recurse subdirs via
`FS_ListFiles(dir,"/",&n)`, then `FS_ListFiles(dir,ext,&n)` appending each
`"dir/file;"` to `results`; `FS_FreeFileList` both. The `_STRINGED` editor branches
(fopen/malloc; `BuildFileList`) are dead (SE-V1).

**Live callers (per-mode: MP engine — server + client of the `jamp` binary).**
- **Inbound game-module trap** — `SP_GETSTRINGTEXTSTRING`
  (`codemp/game/g_public.h:239`), dispatched in `SV_GameSystemCalls`
  (`codemp/server/sv_game.cpp:686-712`): `text = SE_GetString((const char*)VMA(1))`
  (`:699`); if `text[0]` copy into `VMA(2)` via `Q_strncpyz(...,args[3])` and
  return `qtrue`, else copy `"??"` and return `qfalse`. abi-traps row 52.
- **Server direct calls:** `sv_client.cpp:295,302,303,310,311` (challenge/ping
  connect + reject messages, `SE_GetString("MP_SVGAME", …)`); `sv_ccmds.cpp:682`
  (`SE_GetString("STR_SERVER_SERVER_NOT_RUNNING")`); `sv_game.cpp:1746`
  (`Com_Error(ERR_NEED_CD, SE_GetString("CON_TEXT_NEED_CD"))`).
- **Init/frame:** `SE_Init()` from `Com_Init` (`codemp/qcommon/common.cpp:1380`);
  `SE_CheckForLanguageUpdates()` from the client frame (`client/cl_main.cpp:2274`).
- **Client (same MP binary, deferred waves):** `cl_main.cpp:585,1146,2003,2220`,
  `cl_ui.cpp:680,1211,1217,1226`, `cl_cgame.cpp:439,507,1667`,
  `cl_console.cpp:151,575,581`, `win_main.cpp:1496,1505`. Every returned pointer is
  consumed **immediately** (a printf arg / `Q_strcat` / `Q_strncpyz`), one result
  live at a time — so borrowing `&str` at the seam (SE-D1(2)) round-trips.

## State ownership
Mandatory table. Every file-scope global the survey found is listed. Rows below
the rule are external services StringEd **reads/calls**, owned elsewhere and
reached through `EngineHost` (RULING 11/31/36/55) — **all now covered** by the
built 19-method trait (commit b2855df2); no open host gap remains.

| Raven global | oracle cite | Rust owner (crate::Type.field) | constructed by | threaded via |
|---|---|---|---|---|
| `TheStringPackage` (`CStringEdPackage`) | `stringed_ingame.cpp:124` | `stringed::StringEdPackage`, a **field of `Common`** — `mp_engine_qcommon::common::Common.stringed` (SE-D1(1), RULING 2/50; sibling of `modules`, `common.rs:37`), reached as `engine.common.stringed` (`core/src/engine.rs:22`) | **`Engine::new()`'s zeroed-alloc write-list (SE-D6, `core/src/engine.rs:70-96`), NOT a `Common::default()` — none exists.** The whole aggregate is `alloc_zeroed`; then each **not-all-zero-valid** field is `.write()`n in place before `assume_init` (the `//TODO: Port ZeroValid` mechanism, state-ownership.md). `stringed`'s `BTreeMap`/`Vec`/`String` members are NOT zero-valid (an empty `Vec`'s ptr is `NonNull::dangling()`, not null — SE-D1(4)/SE-D6), so it **joins that list**: `addr_of_mut!((*p).common.stringed).write(StringEdPackage::default())`, placed beside the existing `modules`/`time_base` writes (`engine.rs:87,90`) — the identical precedent, not a new mechanism. `StringEdPackage::default()` = Raven ctor's `Clear(SE_FALSE)` (`:79`). | owned; methods take `&self`/`&mut self`; `core`/`server` callers pass `&mut engine.common.stringed` |
| `m_StringEntries` | `stringed_ingame.cpp:62,87` | `BTreeMap<String, SeEntry>` field of `StringEdPackage` (SE-D1(4)) | in-struct, empty | not a separate global |
| `SE_Entry_t` records | `stringed_ingame.cpp:48-59` | `stringed::entry::SeEntry`, owned as `BTreeMap` values | `AddEntry` | by key, never raw ptr (§B5) |
| `m_vstrFlagNames` / `m_mapFlagMasks` | `stringed_ingame.cpp:92-93` | `Vec<String>` + `BTreeMap<String,i32>` fields (SE-D1(4)) | in-struct, empty | not a separate global |
| `se_language` (cvar_t*) | `stringed_ingame.cpp:41`; reg `:1169`; string read `:923`; modified `:1252-1259` | **cvar system** — string value → `EngineHost::cvar_string("se_language")`; `modified` read-and-clear → `cvar_take_modified("se_language")`; registration → `cvar_register("se_language","english",CVAR_ARCHIVE\|CVAR_NORESTART)` (SE-D3, RULING 55) | cvar system | `&mut impl EngineHost` |
| `se_debug` (cvar_t*) | `stringed_ingame.cpp:42`; reg `:1170`; read `:993` | integer read → `EngineHost::cvar_integer("se_debug")`; registration → `cvar_register("se_debug","0",0)` (SE-D3) | cvar system | `&mut impl EngineHost` |
| `sp_leet` (cvar_t*) | `stringed_ingame.cpp:43`; reg `:1171`; read `:756` | integer read → `EngineHost::cvar_integer("sp_leet")`; registration → `cvar_register("sp_leet","0",CVAR_ROM)` (SE-D3) | cvar system | `&mut impl EngineHost` |
| `gvLanguagesAvailable` | `stringed_ingame.cpp:1071` | field of `StringEdPackage` (the language cache; RULING 3 cross-run state → host-struct field, not a Rust global) | `SE_GetNumLanguages` (lazy) | in-struct |
| `giFilesFound` | `stringed_interface.cpp:131` | scratch → an owned return of the list-scan (RULING 3 three-kind; `SE_R_ListFiles` returns `(String, count)`) | list scan | return value, not a global |
| — services below the rule — | | | | |
| `FS_ReadFile` / `FS_FreeFile` | `stringed_interface.cpp:91,119` | qcommon FS | fs init | `EngineHost::fs_read_file` / `fs_free_file` (SE-D3) |
| `FS_ListFiles` / `FS_FreeFileList` | `stringed_interface.cpp:139,158,182-183` | qcommon FS directory scan | fs init | `EngineHost::fs_list_files(dir, ext, want_subs)` (SE-D3, RULING 55; the `ext=="/"` convention lists subdirs) |
| `Com_Printf` / `Com_DPrintf` | `stringed_ingame.cpp:961,1182` | `Common` print state | — | `EngineHost::print` (SE-D3; `Com_DPrintf` = developer-gate + print) |
| `Com_Error` | `stringed_ingame.cpp:957,1190,1257` | error/longjmp model | — | `EngineHost::error(errorParm_t, &str)` (SE-D3) |
| `com_buildScript` (int) | `stringed_ingame.cpp:1176` | cvar system | — | `EngineHost::cvar_integer("com_buildScript")` (SE-D3) |

## Seam definition
StringEd crosses **one** ABI boundary and otherwise exposes engine-internal
(Rust→Rust) functions.

**(a) Inbound game-module syscall arm** — `SV_GameSystemCalls` dispatches
`SP_GETSTRINGTEXTSTRING` (`g_public.h:239`; `sv_game.cpp:686-712`; abi-traps row
52 `trap_SP_GetStringTextString(const char* text, char* buffer, int bufferLength)
-> int`). The arm is Rust→Rust inside the engine, but the `VMA(1)`/`VMA(2)`
pointers cross the VM window. Behavior to reproduce (SE-D1(2)): look the reference
up (borrowing `&str` from the store), then **copy the bytes out** into the caller's
`VMA(2)` buffer capped at `args[3]` exactly as `Q_strncpyz` does — `"??"` +
`qfalse` on empty result, the string + `qtrue` otherwise. The borrow never escapes
the arm, so store-owns / lookup-borrows is observationally identical to Raven's
`const char*` return.

**(b) Engine-internal public API** (frozen signatures; `&self` returns borrow from
the store per SE-D1(2)). The store is `engine.common.stringed: StringEdPackage`.

```rust
impl StringEdPackage {
    // --- lookup (LIVE: serves the trap + sv_client/sv_ccmds/sv_game) ---

    /// `SE_GetString(psPackageAndStringReference)` — ingame.cpp:981-1007.
    /// Copies + uppercases the key internally (owned scratch), finds it, and
    /// returns the debug text when `se_debug->integer && m_bLoadDebug` else the
    /// resolved text; miss → "". Borrows from the entry's owned `String`.
    fn get_string(&self, reference: &str, host: &mut impl EngineHost) -> &str;

    /// `SE_GetString(psPackageReference, psStringReference)` — ingame.cpp:971-978.
    /// Builds "PKG_REF" (owned), uppercases, delegates to `get_string`.
    fn get_string2(&self, package: &str, string_ref: &str,
                   host: &mut impl EngineHost) -> &str;

    /// `SE_GetFlags` — ingame.cpp:1012-1036 (both arities; 2-arg builds the key).
    fn get_flags(&self, reference: &str) -> i32;
    fn get_flags2(&self, package: &str, string_ref: &str) -> i32;

    /// `SE_GetNumFlags` / `SE_GetFlagName` / `SE_GetFlagMask`
    /// — ingame.cpp:1039-1060. `get_flag_name` guards OOR/negative → "" (SE-V4).
    fn get_num_flags(&self) -> i32;
    fn get_flag_name(&self, flag_index: i32) -> &str;
    fn get_flag_mask(&self, flag_name: &str) -> i32;  // `GetFlagMask`, :441-451
}
```

The **trap arm** and **`get_string`/`get_flags*`/flag getters** serve the live
surface and depend only on `EngineHost::cvar_integer` (SE-D3) plus self.
**`get_string`'s `host` param** is required solely for the `se_debug->integer`
debug-branch read. **These arity-suffixed methods are the sole home of Raven's
`SE_GetString`/`SE_GetFlags` C++ overload pairs** (SE-D7/RULING 57;
`get_string2`/`get_flags2` are the 2-arg forms that build `"PKG_REF"` and delegate,
mirroring `ingame.cpp:977,1018`): the intra-binary callers `sv_client.cpp:295-311`
(2-arg → `get_string2`) and `sv_ccmds.cpp:682` (1-arg → `get_string`) — themselves
ported Rust at their server waves — call the methods; **no arity-mangled `SE_*`
free function is re-exported**. The one externally-reached name is the trap arm (a),
`trap_SP_GetStringTextString`.

**Parse-path methods** (`Clear`, `SetupNewFileParse`, `ReadLine`, `ParseLine`,
`AddEntry`, `SetString`, `AddFlagReference`, `CheckLineForKeyword`, `InsideQuotes`,
`ConvertCRLiterals_Read`, `REMKill`, `GetCurrentReference_ParseOnly`,
`ExtractLanguageFromPath`, the `Filename_*`, plus the file-static free functions
`Leetify` / `CopeWithDumbStringData`) are `&mut self`/`&self` internal methods
(the two file-statics are free functions) with **free signatures** (§A1) — their
scratch statics become owned returns / locals (SE-D1(3), RULING 3). The file bytes
arrive as a `&[u8]`/`&str` slice from the caller, so **most need no host service**.
The **exception is the `Leetify` call chain**: `Leetify` reads `sp_leet->integer`
(`:756`), served by `EngineHost::cvar_integer` (SE-D3, state-table `sp_leet` row);
`SetString` calls `Leetify` (`:787`) and `ParseLine` calls `SetString` (`:687,709`).
So `Leetify`, `SetString`, and `ParseLine` each additionally thread
`&mut impl EngineHost` for that one integer read — the mechanical consequence of the
settled `sp_leet` → `cvar_integer` row, not a new host dependency. All other
parse-path methods take no host.

**Load / language-selection API** (Raven `SE_Init`, `SE_Load`, `SE_LoadLanguage`,
`SE_CheckForLanguageUpdates`, `SE_NewLanguage`, `SE_ShutDown`, `SE_GetNumLanguages`,
`SE_GetLanguageName`, `SE_GetLanguageDir`) and the interface TU (Raven
`SE_LoadFileData`, `SE_FreeFileDataAfterLoad`, `SE_BuildFileList`,
`SE_R_ListFiles`) are **internal Rust→Rust functions with idiomatic snake_case
names** (`se_init`, `se_load`, … — SE-D2 as narrowed by SE-D7/RULING 57; not link
targets) and thread `&mut engine.common.stringed` + `&mut impl EngineHost`. **As of
RULING 55 they freeze** — every host service they call now exists in the 19-method
trait (SE-D3): `cvar_register`/`cvar_string`/`cvar_take_modified` (the
`se_language`/`se_debug`/`sp_leet` registration + language-name read + `modified`
update-check) and `fs_list_files` (the directory scan). **Frozen signatures** —
idiomatic names (SE-D2/SE-D7); each function's return type, receiver (`&self` vs
`&mut self`), and `host` presence is **read off** the settled SE-D1(2)/(3) and
SE-V3/V4 (the
return-shape rule after the block), not chosen here:

```rust
// --- ingame.cpp load / language-selection API ---

// SE_Init — ingame.cpp:1156-1196. Registers the three cvars, then loads. void.
fn se_init(pkg: &mut StringEdPackage, host: &mut impl EngineHost);

// SE_ShutDown — ingame.cpp:1198-1201. Clear(SE_FALSE). void, no host.
fn se_shut_down(pkg: &mut StringEdPackage);

// SE_NewLanguage — ingame.cpp:1144-1147. Clear(SE_TRUE). void, no host.
fn se_new_language(pkg: &mut StringEdPackage);

// SE_Load — ingame.cpp:910-966. Prepends "strings/<lang>/" via cvar_string.
// Error message (va() scratch, SE-D2) → owned String; NULL-for-ok → None.
fn se_load(pkg: &mut StringEdPackage, file_name: &str, load_debug: bool,
           fail_is_critical: bool, host: &mut impl EngineHost) -> Option<String>;

// SE_Load_Actual — ingame.cpp:828-873 (file-static). SE_LoadFileData → parse loop.
// Same error-message-or-None return as se_load.
fn se_load_actual(pkg: &mut StringEdPackage, file_name: &str, load_debug: bool,
                  speculative_load: bool, host: &mut impl EngineHost) -> Option<String>;

// SE_LoadLanguage — ingame.cpp:1208-1243. SE_NewLanguage + build-list + SE_Load each
// matching file; threads the SE_Load error up (→ None on ok).
fn se_load_language(pkg: &mut StringEdPackage, language: &str, load_debug: bool,
                    host: &mut impl EngineHost) -> Option<String>;

// SE_CheckForLanguageUpdates — ingame.cpp:1250-1261. cvar_take_modified collapses
// Raven's `if (se_language->modified) { …; ->modified = SE_FALSE; }` into one call.
fn se_check_for_language_updates(pkg: &mut StringEdPackage, host: &mut impl EngineHost);

// SE_GetNumLanguages — ingame.cpp:1072-1116. Lazily populates the language cache
// (→ &mut) via fs_list_files; returns the cache size.
fn se_get_num_languages(pkg: &mut StringEdPackage, host: &mut impl EngineHost) -> i32;

// SE_GetLanguageName — ingame.cpp:1120-1129. Returns c_str() of a STORED cache entry
// → borrow &str (SE-D1(2)); OOR/negative → "" (SE-V3/V4, no assert). Read-only, no host.
fn se_get_language_name(pkg: &StringEdPackage, lang_index: i32) -> &str;

// SE_GetLanguageDir — ingame.cpp:1133-1142. Returns a va("%s/%s") SCRATCH string
// → owned String (SE-D1(3)); OOR/negative → "" (SE-V3/V4). Read-only, no host.
fn se_get_language_dir(pkg: &StringEdPackage, lang_index: i32) -> String;

// SE_GetFoundFile — ingame.cpp:875-902 (file-static). Consumes one ';'-delimited
// entry from the in/out results string (erases it in place), returns the extracted
// name; empty results → Raven's NULL loop-terminator (:880) → None. No host.
fn se_get_found_file(results: &mut String) -> Option<String>;

// --- interface.cpp TU (in-engine #ifndef _STRINGED paths, SE-V1) ---

// SE_LoadFileData — interface.cpp:41-104. FS_ReadFile; return shape forced by the
// state-table host row (fs_read_file). Raven's optional length out-param → Vec::len.
fn se_load_file_data(file_name: &str, host: &mut impl EngineHost) -> Option<Vec<u8>>;

// SE_FreeFileDataAfterLoad — interface.cpp:109-122. FS_FreeFile; the owned buffer's
// free collapses into fs_free_file (Vec-by-value, state-table row).
fn se_free_file_data_after_load(data: Vec<u8>, host: &mut impl EngineHost);

// SE_BuildFileList / SE_R_ListFiles — interface.cpp:132-212. fs_list_files(dir,"/",_)
// lists subdirs; fs_list_files(dir,".str",_) lists files. giFilesFound → the returned
// count (RULING 3, state-table row); the ';'-delimited accumulator → the returned String.
// SE_R_ListFiles keeps Raven's (extension, path) arg order.
fn se_build_file_list(dir: &str, host: &mut impl EngineHost) -> (String, i32);
fn se_r_list_files(ext: &str, dir: &str, host: &mut impl EngineHost) -> (String, i32);
```

**Return-shape rule (no new decision — read off SE-D1/SE-D2/SE-V3/V4).** A getter
that returns a pointer into **stored** package storage borrows `&str` (SE-D1(2):
`se_get_language_name` → the cache `Vec`); one that returns a `va()`/`static`-scratch
string returns an owned `String` (SE-D1(3)/RULING 3: `se_get_language_dir`). Raven's
`LPCSTR` error returns (`NULL`=ok) become `Option<String>` because the message is a
`va()` scratch (SE-D2). The OOR arms of `se_get_language_name`/`se_get_language_dir`
return `""` — **not** `None`, **not** a panic — because SE-V3/V4 pin the faithful
observable as the fall-through empty string (the release `assert` is a no-op);
`se_get_found_file` alone returns `None`, because Raven's sentinel there is a literal
`NULL` loop-terminator (`:880`), not `""`. `&self` vs `&mut self` follows whether the
body mutates: `se_get_num_languages` lazily fills the cache (`&mut`), the two language
getters only read (`&self`). A `host` param appears iff the body calls a host service
(`se_new_language`/`se_shut_down` and the two language getters call none).

## Decisions
Rendered from RULING 50 and its cross-cutting rulings (RULING 3/40/32/44/2), then
folded forward to **RULING 55** (SE-D3 extension, SE-D5 span, SE-D6 construction)
and **RULING 57** (SE-D7 lookup-API naming; SE-D2 narrowing).

- **SE-D1.** RULING 50's four-part faithful set:
  1. **Ownership** — the package is a **`Common` sub-struct field**
     (`engine.common.stringed`), threaded `&mut`, **no singleton, no `static mut`**
     (fork-2 / RULING 2 precedent). Because §B forbids Rust globals and RULING 50
     fixes the home as `Common`. Its construction is SE-D6. Rejected a Rust
     `static`/`OnceCell` (§B3) and a `core::Engine` top-level field (RULING 50 says
     `Common`).
  2. **Seam returns** — the store owns `String`s; lookups **borrow `&str`** from
     it; the **syscall arm copies bytes out** exactly as Raven's `Q_strncpyz` trap
     does. Raven's returned `const char*` points into map-entry storage stable
     until the next language reload (`Clear`); every live caller consumes it
     immediately (Raven ground truth), so a borrow that never escapes the call is
     observationally identical. Rejected returning an owned `String` at every
     lookup (a needless copy on the hot trap) and returning a raw pointer (§B5).
  3. **Filename_* / scratch statics** — `Filename_PathOnly`/`WithoutExt`/
     `WithoutPath`, `InsideQuotes`, `ConvertCRLiterals_Read`, `Leetify` follow
     RULING 3's three-kind rule: rotating scratch/return buffers → **owned
     returns** (or caller locals). Faithful because no caller holds two results of
     the same helper simultaneously (Raven ground truth: the nesting chains
     *distinct* helpers, each with its own static, and copies the inner result out
     before the next call). Rejected keeping process-wide mutable static buffers
     (§B3) and a shared scratch field (would reintroduce the aliasing the analysis
     just ruled out).
  4. **Containers** — `map<string,SE_Entry_t>` → **`BTreeMap<String,SeEntry>`**
     (keeps Raven's sorted `std::map` iteration order); `vector<string>` flag names
     → `Vec<String>`; `map<string,int>` flag masks → `BTreeMap<String,i32>`.
     Because SE-D1(4)/RULING 50 pins the ordered map even though this TU only
     `find()`s it (determinism + faithful parity). Rejected `HashMap`
     (non-deterministic order; diverges from the ruling).
- **SE-D2.** Naming per RULING 40 (**narrowed by SE-D7/RULING 57**):
  `CStringEdPackage` → **`StringEdPackage`** (bare `C` prefix drops); `SE_Entry_s`
  is **internal** (`std::string` members, NOT ABI-frozen) so it becomes **`SeEntry`**
  per the same rule; the **engine-side `SE_*` functions are internal Rust→Rust and
  take idiomatic snake_case names** (`se_init`, `se_load`, …; the lookup overload
  pairs become the `get_string`/`get_flags` seam methods, SE-D7). **Only** the
  genuinely externally-reached surface keeps a verbatim Raven-derived name — the
  game-module trap arm `trap_SP_GetStringTextString` (abi-traps row 52) and any true
  link target. Because RULING 40 drops the hungarian prefix for §F internals and
  keeps only genuine ABI/link names verbatim. Rejected freezing `SE_Entry_s` as an
  ABI name (it has no fixed layout — `std::string`) and the earlier reading that the
  engine-side `SE_*` C++ functions are "syscall/link targets" (SE-D7 corrects this:
  they are internal calls the ported Rust callers reach as methods/free functions,
  not link symbols).
- **SE-D3.** Upward services reached through the **one built `EngineHost`**
  (`crates/mp/host-interface/src/engine_host.rs`, RULING 31/33/36 and **extended by
  RULING 55**). As of RULING 55 the trait is **19 methods, built** (commit
  b2855df2): the RULING-36 base (trace/FS read+free/print/error/`VM_Call`/shared
  memory/`flrand`/`irand`/gentity/`cvar_integer`/`sv_time`/FS write/model mdxm+mdxa)
  **plus the four StringEd-closing methods** — `cvar_register`, `cvar_string`,
  `cvar_take_modified`, `fs_list_files` — which close SE-Q1/SE-Q2. Because RULING 11
  mandates the one services trait and RULING 55 built exactly the surface StringEd's
  live boot path needs. Rejected a bespoke StringEd services trait (RULING 11) and
  an alternate out-of-trait threading of the language name (RULING 55 chose the
  trait extension). **The full 19-method trait, verbatim from
  `engine_host.rs:24-194`** (doc-comment source cites elided for the four
  pre-existing FS/model methods; the four new methods carry their RULING-55 rationale
  in full):

  ```rust
  pub trait EngineHost {
      fn trace(
          &mut self,
          results: &mut trace_t,
          start: &vec3_t,
          mins: &vec3_t,
          maxs: &vec3_t,
          end: &vec3_t,
          pass_entity_num: i32,
          contentmask: i32,
          capsule: bool,
          trace_flags: i32,
          use_lod: i32,
      );
      fn fs_read_file(&mut self, qpath: &str) -> Option<Vec<u8>>;
      fn fs_free_file(&mut self, _buffer: Vec<u8>) {}
      fn print(&mut self, msg: &str);
      fn error(&mut self, code: errorParm_t, msg: &str) -> !;
      fn vm_call(&mut self, vm: VmSlot, callnum: i32, args: &[isize]) -> isize;
      fn shared_memory(&mut self) -> *mut c_char;
      fn flrand(&mut self, min: f32, max: f32) -> f32;
      fn irand(&mut self, min: i32, max: i32) -> i32;
      fn gentity(&mut self, ent_num: i32) -> *mut sharedEntity_t;
      fn cvar_integer(&mut self, name: &str) -> i32;
      fn sv_time(&mut self) -> i32;
      fn fs_write_file(&mut self, qpath: &str, data: &[u8]) -> bool;
      fn model_mdxm(&mut self, model: qhandle_t) -> *mut c_void;
      fn model_mdxa(&mut self, model: qhandle_t) -> *mut c_void;

      // --- RULING 55: the four StringEd-closing methods ---

      /// Raven `Cvar_Get`'s registration side (ruling 55) — establish the cvar
      /// with `default` exactly once (creation sets string=default, integer=
      /// atoi, `modified = qtrue`, cvar.cpp:261-273); an already-existing cvar
      /// keeps its value and only ORs `flags` in (cvar.cpp:209-232). The
      /// returned `cvar_t*` collapses away — reads go through the by-name
      /// services. StringEd registers `se_language`/`se_debug`/`sp_leet` this
      /// way in SE_Init.
      /// Source: `oracle/codemp/qcommon/cvar.cpp:188` (SE_Init sites:
      /// `oracle/codemp/qcommon/stringed_ingame.cpp:1169-1171`)
      fn cvar_register(&mut self, name: &str, default: &str, flags: i32);

      /// Per-call string cvar read (ruling 55) — collapses Raven's cached
      /// `cvar_t->string` reads (SE_Load's `se_language->string` path build,
      /// `stringed_ingame.cpp:921-925`). A missing name reads `""`, as
      /// `Cvar_VariableString` returns.
      /// Source: `oracle/codemp/qcommon/cvar.cpp:133-140`
      fn cvar_string(&mut self, name: &str) -> String;

      /// Read-and-clear of Raven's `cvar_t->modified` flag (ruling 55): returns
      /// the flag and clears it in the same call — Raven's two-step update-check
      /// idiom (`if (se_language->modified) { ...; se_language->modified =
      /// SE_FALSE; }`, SE_CheckForLanguageUpdates) collapsed so no host
      /// round-trip can observe the in-between state. A missing name reads
      /// `false`.
      /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1252-1259`
      fn cvar_take_modified(&mut self, name: &str) -> bool;

      /// Raven `FS_ListFiles` + `FS_FreeFileList` collapsed (ruling 55) — the
      /// VFS/pk3-aware listing over the FS search paths, DISTINCT from
      /// `PlatformHost::list_files` (`Sys_ListFiles`, a raw OS directory scan):
      /// this one sees pak contents. Subdirectories are requested with
      /// `ext = "/"` (`SE_R_ListFiles`, `stringed_interface.cpp:139`), files by
      /// extension (`:158`); the free collapses into the `Vec` drop
      /// (`:182-183`). `want_subs` extends the match into subdirectories (the
      /// ruled surface; Raven's own `FS_ListFiles` is 3-param, `files.cpp:2174`
      /// — today's call sites pass `false`).
      /// Source: `oracle/codemp/qcommon/files.cpp:2174`
      fn fs_list_files(&mut self, dir: &str, ext: &str, want_subs: bool) -> Vec<String>;
  }
  ```

  StringEd's method-to-service mapping: file read (`SE_LoadFileData`) →
  `fs_read_file` + `fs_free_file`; prints (`Com_Printf`/`Com_DPrintf`) → `print`;
  fatal (`Com_Error`) → `error`; integer cvars (`se_debug`/`sp_leet`/
  `com_buildScript`) → `cvar_integer`; cvar registration (`SE_Init`) →
  `cvar_register`; `se_language->string` (`SE_Load` path build, `Language_Is*`) →
  `cvar_string`; `se_language->modified` (`SE_CheckForLanguageUpdates`) →
  `cvar_take_modified`; directory scan (`SE_R_ListFiles`) → `fs_list_files`.
  **MockHost's cvar registry drives the goldens (RULING 32/55):** the string value
  is the single source of truth and `cvar_integer` derives from it via `atoi`
  semantics, so `cvar_register`/`cvar_string`/`cvar_integer`/`cvar_take_modified`
  are internally consistent under test. `fs_list_files`'s `want_subs` has no Raven
  counterpart (today's call sites pass `false`); `fs_list_files` is DISTINCT from
  `PlatformHost::list_files` (VFS/pk3-aware vs raw OS scan).
- **SE-D4.** Verification per §18 — a `tools/stringed-oracle/` harness (gp2-oracle
  pattern) compiles the **unmodified** `stringed_ingame.cpp` + `stringed_interface.cpp`
  standalone, driven by **hand-authored `.str`/`.ste` fixtures** (no retail data),
  with goldens pinning load/parse/lookup/flag-mask behavior including the flag-bit
  encounter order and the SE-D1(2) reference-stability semantics. Because §18
  requires the differential proof against the oracle TU and RULING 32 forbids
  retail blobs. Rejected clang-layout-only checks (no layout to assert — §F types
  are layout-free) and a live-only check (the differential golden is the §F gate).
  **Link-set confirmed:** both TUs are in `WinDed.vcproj:258-267` (Scope above).
- **SE-D5.** RULINGS 11-56 stand (RULING 57 is the closing addition). RULING 44
  (Win32 `long` in a binary/file format = 4 bytes) is noted but **not applicable
  here**: the `.str`/`.ste` files are line-oriented ASCII text
  (`ReadLine`/`ParseLine`), with **no binary `long` field** — StringEd has no
  on-disk fixed-width struct (contrast ROFF/nav). RULING 3 (statics three-kind) is
  applied in SE-D1(3); RULING 2 (`Engine`/`Common` sub-structs) in SE-D1(1)/SE-D6.
- **SE-D6.** **Construction story (RULING 55 correction, mechanical, evidence-forced).**
  `common.stringed` is written **explicitly** in `Engine::new()`'s zeroed-alloc
  write-list (`core/src/engine.rs:70-96`) —
  `addr_of_mut!((*p).common.stringed).write(StringEdPackage::default())` — **not**
  via a `Common::default()`, because **there is no `Common::default()`** (`Common`
  is built only through `Engine::new`'s `alloc_zeroed` + explicit-write path,
  state-ownership.md STATE-D5/LIFE-Q9). The write is forced because
  `StringEdPackage`'s `BTreeMap`/`Vec`/`String` fields are **not** all-zero-valid
  (an empty `Vec`'s pointer is `NonNull::dangling()`, not null), exactly the
  existing `modules`/`time_base` precedent (`engine.rs:87,90`);
  `StringEdPackage::default()` = Raven ctor's `Clear(SE_FALSE)` (`:79`). Because a
  zeroed niche is UB to `assume_init` for these collections (the `//TODO: Port
  ZeroValid` hazard, build-out plan §3d). Rejected relying on a zero-fill (UB) and
  inventing a `Common::default()` (it does not exist; would fork the settled
  construction convention).
- **SE-D7.** **The arity-overloaded lookup API (RULING 57, 2026-07-09, closes
  SE-Q3).** Raven's C++ overload pairs — `SE_GetString(psPackageReference,
  psStringReference)` / `SE_GetString(psPackageAndStringReference)`
  (`stringed_ingame.cpp:971,981`) and `SE_GetFlags(...,...)` / `SE_GetFlags(...)`
  (`:1012,1021`) — exist **ONLY** as the seam's `StringEdPackage` methods, arity
  encoded in the name exactly as **Seam definition (b)** already froze them:
  `get_string`/`get_string2`, `get_flags`/`get_flags2` (each 2-arg form builds
  `"PKG_REF"`, uppercases, and delegates to its 1-arg sibling, mirroring
  `:977,1018`). The intra-binary callers — `sv_client.cpp:295-311` (2-arg,
  `Com_Printf(SE_GetString("MP_SVGAME", …))` → `get_string2`) and `sv_ccmds.cpp:682`
  (1-arg, `SE_GetString("STR_SERVER_SERVER_NOT_RUNNING")` → `get_string`) — are
  themselves ported to Rust at their own server waves and **call the methods**, not a
  re-exported C name. The one genuinely externally-reached name is the game-module
  trap arm `trap_SP_GetStringTextString` (abi-traps row 52), specified in **Seam
  definition (a)**. Because RULING 40 mandates idiomatic naming for internals and the
  seam is authoritative — a C++ overload set has no single C link symbol to preserve,
  and every engine-side caller is itself Rust. Rejected re-exporting arity-mangled
  `SE_GetString`/`SE_GetFlags` free functions (no link target needs them; would fork
  the Seam's already-frozen method names). This narrows SE-D2's earlier
  "syscall/link targets" rationale to the trap arm alone.

## Verification strategy
Per DEC-09 and porting-rules **§F (rules 18-20)** — C++-track, differential against
the unmodified oracle TUs, goldens committed so `cargo test` needs no C++
toolchain.

- **Harness**: `tools/stringed-oracle/` compiles the unmodified
  `codemp/qcommon/stringed_ingame.cpp` **+** `stringed_interface.cpp` standalone
  under stub headers (mirroring `tools/gp2-oracle/`), built with the WinDed macro
  set (`-DNDEBUG -DDEDICATED`, `_STRINGED` undefined — so the SE-V1/V3 branches
  match the ported build), stubbing the seam behind a deterministic host
  (`FS_ReadFile`/`FS_FreeFile`, `FS_ListFiles`/`FS_FreeFileList`, `Com_Printf`,
  `Cvar_*`) to capture behavior. Rust side = **MockHost with the cvar registry
  (RULING 32/55): the cvar string is the single source of truth, `cvar_integer`
  derives via `atoi`, `cvar_register` seeds default+`modified`, `cvar_take_modified`
  reads-and-clears** — so the four RULING-55 methods are exercised consistently.
- **Fixture set (hand-authored, SE-D4/RULING 32)**: `.str` files exercising every
  `ParseLine` keyword (VERSION good/bad, CONFIG/FILENOTES/NOTES absorb, REFERENCE,
  FLAGS with ≥2 flags across ≥2 references to pin bit assignment, ENDMARKER,
  `LANG_ENGLISH` with `\n` literals, hi-char bytes for CopeWithDumbStringData,
  `//`-comments inside and outside quotes for REMKill), a truncated file (missing
  ENDMARKER), and a matching `.ste` override (foreign + `#same`). A `strings/<lang>/`
  tree for `SE_BuildFileList`/language enumeration. Committed; **no retail blobs**.
- **Golden A — parse/lookup**: after loading a fixture, dump every `m_StringEntries`
  key→(`m_strString`, `m_strDebug`, `m_iFlags`) pair, the `m_vstrFlagNames` order +
  `m_mapFlagMasks` values, and a battery of `SE_GetString`/`SE_GetFlags` results
  (hit, miss→"", debug-branch on vs off, 2-arg key build, uppercase folding). Rust
  reproduces byte-for-byte, **including the BTreeMap-visible ordering** (SE-D1(4)).
- **Golden B — reference stability + language reload**: capture a `SE_GetString`
  result, then `SE_NewLanguage`/reload, and confirm the flag tables survive the
  `Clear(SE_TRUE)` while entries are rebuilt (Clear's `!bChangingLanguages`
  semantics, `:131-144`) — pins SE-D1(2)'s stability contract and the flag-cache
  invariant. Also drives `SE_CheckForLanguageUpdates` through `cvar_take_modified`
  (modified→reload→cleared, SE-D3).
- **Golden C — file-list scan**: `SE_BuildFileList` over the fixture `strings/`
  tree → the exact `';'`-delimited result string and `giFilesFound` count, and the
  `SE_GetNumLanguages` dedup/english-first ordering. Driven by the MockHost
  `fs_list_files` (SE-D3, RULING 55) — the `ext=="/"` subdir vs `".str"` file
  convention pinned.
- **Live tie-in**: once the server spine + `SV_GameSystemCalls` land, the
  `SP_GETSTRINGTEXTSTRING` arm and the `sv_client`/`sv_ccmds`/`sv_game` direct
  calls fall out of the whole-syscall A/B referee diff.
- **UB / dead surface (§19/§20)**: SE-V1/V2/V6 drops carry ≤2-line notes and are
  absent from the fixtures; SE-V3's release-`assert` no-op is matched by building
  the oracle TU `-DNDEBUG`; SE-V4's signed/unsigned index compare is reproduced by
  an explicit negative-index guard and pinned by an OOR/negative fixture case.

## Slice hooks
From `docs/plans/2026-07-08-mp-engine-build-out.md`:
- **Stage 0 (interface crate)** exposes the SE-D3 methods StringEd calls
  (`fs_read_file`, `fs_free_file`, `print`, `error`, `cvar_integer`,
  `cvar_register`, `cvar_string`, `cvar_take_modified`, `fs_list_files` — **all
  present** in the 19-method trait, commit b2855df2) before the whole subsystem
  compiles. The lookup seam (Golden A) needs only the store + `cvar_integer`.
- **`Com_Init` (init wave)** calls `SE_Init` (`common.cpp:1380`) — needs the load
  path, which is now fully unblocked (SE-D3 provides `cvar_register`/`cvar_string`/
  `fs_list_files`; RULING 55 closed the former SE-Q1/SE-Q2 gaps).
- **`SV_GameSystemCalls` (server wave)** dispatches the `SP_GETSTRINGTEXTSTRING`
  arm — needs the trap arm + `get_string` frozen (they are).
- **Client frame** calls `SE_CheckForLanguageUpdates` (`cl_main.cpp:2274`) — needs
  `cvar_take_modified` (present, SE-D3).
- **First-slice skeleton boundary (dry-run note).** A porter produces, with **no
  open points**: `mod.rs` consts + re-exports; `entry.rs` `SeEntry`; the whole
  parse path, split across two files **per the frontmatter `files:` roster (the
  authoritative machine-consumed placement, doc-standards rule 6)**:
  - On **`package.rs`** (the `StringEdPackage` member methods): `Clear`,
    `SetupNewFileParse`, `ReadLine`, `REMKill`, `CheckLineForKeyword`,
    `InsideQuotes`, `ConvertCRLiterals_Read`, `ParseLine`, `AddEntry`, `SetString`,
    `AddFlagReference`, `GetFlagMask`, `GetCurrentReference_ParseOnly`
    (`:731`, called by `ParseLine` `:617,653`), `ExtractLanguageFromPath`
    (`:230`, called by `SetupNewFileParse` `:244`), the `Filename_*`.
  - On **`api.rs`** (the file-static free functions, not class members —
    `stringed_ingame.cpp:752,483`): `Leetify`, `CopeWithDumbStringData`.

  plus the frozen lookup seam (`get_string`, `get_flags*`, flag getters).

  **The `SP_GETSTRINGTEXTSTRING` copy-out arm is NOT first-slice output** — it is
  deferred to the **wave-20 server slice**. Its home is `sv_game_system_calls`
  (Raven `SV_GameSystemCalls`, `sv_game.cpp:686-712`) in **`mp_engine_server`**
  (`crates/mp/engine/server/src/server_host.rs`), a different crate that this
  doc's frontmatter `files:` roster — the authoritative machine-consumed placement
  (doc-standards rule 6) — deliberately does **not** list (all five roster files
  are `mp_engine_qcommon/stringed/*`). `SV_GameSystemCalls` sits at wave 20 and the
  `EngineHostView` trait impl lands with it (engine-fork-discovery **RULING 43**,
  impl-at-wave-20; build-out plan §M4), consistent with **Slice hooks** bullet 3
  above. The first slice produces only the frozen lookup seam the arm will call
  (`get_string`); the arm's copy-out behavior is specified in **Seam definition (a)**
  for the wave-20 porter to transcribe there, not here.

  **Wiring `common.stringed` (no open point — mechanical, off the tree, SE-D6).**
  The one new piece of shared state this doc adds is a `pub stringed:
  StringEdPackage` field on `Common` (`common/common.rs:37`, sibling of `modules`).
  Because `Common` is built through `Engine::new()`'s `alloc_zeroed` +
  explicit-write path (SE-D6; state-table "constructed by"; `core/src/engine.rs:70-96`)
  and `StringEdPackage`'s `BTreeMap`/`Vec`/`String` fields are not all-zero-valid,
  the porter **extends that function's existing write-list** with
  `addr_of_mut!((*p).common.stringed).write(StringEdPackage::default())`, placed
  next to the `modules`/`time_base` writes (`engine.rs:87,90`) — the same mechanism
  those fields already use, not a new one. (`StringEdPackage::default()` = Raven
  ctor's `Clear(SE_FALSE)`, SE-D1(1)/SE-D6.) There is **no** `Common::default()`.
  This is the ZeroValid niche hazard the build-out plan §3d warns about; the
  explicit `.write()` is the settled cure.

  **File placement note:** `Leetify`/`CopeWithDumbStringData` live in `api.rs` yet
  are **first-slice** — they sit on the live parse path (`ParseLine` calls both)
  and depend only on `cvar_integer` (SE-D3) + owned buffers. The whole
  load/enumeration API (`SE_Init`, `SE_Load`, `SE_LoadLanguage`,
  `SE_CheckForLanguageUpdates`, `SE_NewLanguage`, `SE_ShutDown`,
  `SE_GetNumLanguages`, `SE_GetLanguageName`/`Dir`, `SE_Load_Actual`,
  `SE_GetFoundFile`) plus the interface TU is **also first-slice** now that its host
  services all exist (SE-D3); nothing in this subsystem is gated on an unresolved
  host method. The §20 drops (SE-V1/V2/V6) are notes, not stubs.

## Open questions
None. All three prior open questions are closed:
- **SE-Q1** (se_language cvar string/`modified` access + `Cvar_Get` registration) —
  CLOSED by **RULING 55** (2026-07-09): `EngineHost::cvar_string`,
  `cvar_take_modified`, `cvar_register` (SE-D3, commit b2855df2).
- **SE-Q2** (FS directory listing for `SE_R_ListFiles`) — CLOSED by **RULING 55**
  (2026-07-09): `EngineHost::fs_list_files(dir, ext, want_subs)` (SE-D3, commit
  b2855df2).
- **SE-Q3** (the arity-overloaded lookup API — do Raven's `SE_GetString`/`SE_GetFlags`
  C++ overload pairs keep exact Raven names, or become idiomatic seam methods?) —
  CLOSED by **RULING 57** (2026-07-09): idiomatic `StringEdPackage` methods
  `get_string`/`get_string2`/`get_flags`/`get_flags2`; only the game-module trap arm
  keeps a Raven-derived name (SE-D7, SE-D2).
