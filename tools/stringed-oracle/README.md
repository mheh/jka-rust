# stringed-oracle — differential golden harness for the StringEd port

Verifies the `mp_engine_qcommon::stringed` §F reimplementation
(`docs/subsystems/stringed.md`, Status: REVIEWED) against the **unmodified**
Raven StringEd TUs, exactly like `tools/gp2-oracle` and `tools/icarus-oracle`
(porting-rules §18). The two oracle `.cpp` are compiled standalone against stub
headers; canonical dumps live under `goldens/` and are committed, so the Rust
parity tests need **no** C++ toolchain — only `build.sh` does, to (re)generate
or check.

`oracle/` is never edited.

## Usage

```sh
sh build.sh          # build the dumper, run 3 modes, diff against goldens/
sh build.sh --regen  # regenerate goldens/*
```

Toolchain: Homebrew `g++-16` (override with `CXX=`). Oracle-parity flags match
the sibling harnesses (`-fsigned-char -ffp-contract=off -fno-fast-math`), plus
the WinDed **Release macro set** the ported build models (stringed.md appendix):
`-DNDEBUG` (so the SE-V3 `__ASSERT` calls are no-ops → faithful fall-through
returns), `-DDEDICATED`, and `_STRINGED` **undefined** (SE-V1 editor branches
compile out). `-std=c++14 -w`.

## Harness shape (how it drives the unmodified oracle)

`CStringEdPackage` and its `TheStringPackage` global live **inside**
`stringed_ingame.cpp` with no header (unlike GP2/ICARUS, whose classes are
header-public). To enumerate `m_StringEntries` in `std::map` (→ Rust `BTreeMap`)
sorted order, `dump.cpp` **`#include`s the two byte-for-byte oracle TUs** into
one dumper TU — still a standalone compile of the unmodified source (§18), just
aggregated so internal state is visible. `host.cpp` supplies the engine seam as
a deterministic **MockHost twin** (RULING 32/55): an in-memory cvar registry
where the cvar *string* is the single source of truth and `integer` derives via
`atoi` (`cvar_register`/`cvar_string`/`cvar_integer`/`cvar_take_modified` stay
consistent), and a fixture-backed VFS. `FS_ListFiles` **sorts** its results
(readdir order is FS-dependent) so the scan is run-twice byte-identical.

## The three goldens (each maps to a doc § Verification-strategy unit)

| Golden | Pins | Doc unit |
| --- | --- | --- |
| `goldens/parse_lookup.txt` | Load english, then dump every `m_StringEntries` key→(`m_strString`,`m_strDebug`,`m_iFlags`) in **BTreeMap-sorted** order (SE-D1(4)); the `m_vstrFlagNames` order + `m_mapFlagMasks` values (first-seen encounter-order bit assignment, `AddFlagReference`); a lookup battery — hit, uppercase-fold, 2-arg key build, CR-literal (`\n`), hi-char `CopeWithDumbStringData`, miss→`""`, `SE_GetFlags` miss→0 (SE-V3), `SE_GetFlagName` OOR/negative→`""` (SE-V4); the `se_debug=1` debug-branch (`m_strDebug`); a `sp_leet=42` `Leetify` reload; and the four `ParseLine`/load error messages (truncated, bad VERSION, unknown keyword, missing file). | **Golden A — parse/lookup** |
| `goldens/reference_stability.txt` | Capture a lookup, `SE_NewLanguage` (`Clear(SE_TRUE)`) → flag **name table survives** while entries clear; reload german (`.ste` override + `#same`→cached-english) with masks persisting but rebuilt entries carrying flags 0; `SE_CheckForLanguageUpdates` via `cvar_take_modified` (modified→reload→cleared, second call no-op); `SE_ShutDown` (`Clear(SE_FALSE)`) → flags cleared. Pins SE-D1(2) stability + the `Clear` `!bChangingLanguages` invariant (`:131-144`). | **Golden B — reference stability + reload** |
| `goldens/filelist_scan.txt` | `SE_BuildFileList("strings")` → exact `';'`-delimited result + count (the `ext=="/"` subdir vs `".str"` file convention, `SE_R_ListFiles`); `SE_GetNumLanguages` dedup + **english-first** ordering; `SE_GetLanguageName`/`Dir` per index, plus OOR/negative→`""` (SE-V3/V4). Driven by the MockHost `fs_list_files`. | **Golden C — file-list scan** |

The Rust port's parity test (in `mp_engine_qcommon`, once landed) reads
`fixtures/*` + `goldens/*` from here and must reproduce every golden exactly.

## Fixture corpus (hand-authored, SE-D4/RULING 32 — no retail data)

Under `fixtures/strings/<lang>/` (the enumeration scan tree) plus `fixtures/misc/`
(error fixtures, deliberately outside the scan tree so language loads stay clean):

- `strings/english/menus.str` — second package (`MENUS_*`), plain refs, no flags.
- `strings/english/objectives.str` — every `ParseLine` keyword: VERSION,
  CONFIG/FILENOTES/NOTES (absorbed), REFERENCE, **FLAGS** (2 flags on MISSION01 +
  1 on MISSION02 → bits 0,1,2 pin encounter order), ENDMARKER, `LANG_ENGLISH`
  with a `//`-comment **inside** quotes (kept) and **outside** (stripped by
  REMKill), a `\n` CR-literal, and raw hi-char bytes
  (`0x92 0x93 0x94 0x0b 0x85 0x96 0x97`, `?.`, tab) for `CopeWithDumbStringData`.
- `strings/french/objectives.str` — foreign, exercises `CopeWith(FRENCH)`.
- `strings/german/objectives.str` + `.ste` — foreign english-master + foreign +
  `#same`, with a speculative `.ste` override; drives multi-language enumeration.
- `misc/{truncated,badversion,unknownkw}.str` — the three parse/load error paths.

## Normalizations

**None.** Both unmodified oracle TUs compile clean under the stub headers on this
host (LP64/macOS); no source edits or `perl` repairs are applied to the `build/`
copies. RULING 44 (Win32 `long`=4 bytes in binary formats) is **not applicable**:
`.str`/`.ste` are line-oriented ASCII text with no on-disk fixed-width field
(stringed.md SE-D5).

## Deviations from the doc's plan (with reason)

1. **The oracle TUs are `#include`d into the dumper, not linked separately.**
   GP2/ICARUS link and drive a header-public class; StringEd's class is
   TU-private, so enumerating `m_StringEntries` (Golden A's core, incl. the
   BTreeMap ordering the doc requires) needs TU-level visibility. The source is
   byte-unmodified — still a standalone §18 compile.
2. **The truncated probe is preceded by `SE_NewLanguage()`.** The oracle resets
   the end-marker flag only in `Clear()`, not `SetupNewFileParse` — a prior
   file's ENDMARKER would mask a later truncation. Normal flow (`SE_LoadLanguage`
   → `SE_NewLanguage`) resets it first; the dumper reproduces that reset so the
   truncation error the fixture exists to pin actually fires (faithful quirk
   noted at the call site).

## Determinism / size

Run-twice byte-identical (verified per mode; `FS_ListFiles` sorts). Committed:
3 goldens + 8 fixtures + `dump.cpp` + `host.cpp` + 3 stub headers + `build.sh`
+ `.gitignore` = 18 files, ~27 KB (`build/` is git-ignored).
