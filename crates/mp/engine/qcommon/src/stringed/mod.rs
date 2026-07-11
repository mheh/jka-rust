//! StringEd localization package (`CStringEdPackage`) — MP engine.
//!
//! C++-track idiomatic reimplementation (porting-rules §F) of Raven's StringEd
//! localization subsystem, spanning the two linked TUs
//! `oracle/codemp/qcommon/stringed_ingame.{h,cpp}` and
//! `oracle/codemp/qcommon/stringed_interface.cpp`. Design frozen in
//! `docs/subsystems/stringed.md`.
//!
//! - `CStringEdPackage` → [`package::StringEdPackage`] (renamed per SE-D2 /
//!   RULING 40 bare-`C`-prefix drop) — the localized-string store + parse
//!   scratch + flag tables; a field of `Common` (`engine.common.stringed`,
//!   SE-D1(1)/SE-D6), no singleton, no `static mut`.
//! - `SE_Entry_s` → [`entry::SeEntry`] (internal, `std::string` members, not ABI).
//! - The load/enumeration `SE_*` C API + the file-static helpers (`Leetify`,
//!   `CopeWithDumbStringData`, `SE_Load_Actual`, `SE_GetFoundFile`) →
//!   idiomatic snake_case free functions in [`api`] (SE-D2/SE-D7: internal
//!   Rust→Rust, not link/syscall targets).
//! - The engine-side interface TU (`SE_LoadFileData`,
//!   `SE_FreeFileDataAfterLoad`, `SE_BuildFileList`, `SE_R_ListFiles`) → [`interface`].
//!   Only the in-engine `#ifndef _STRINGED` branches port; the `_STRINGED`
//!   editor-tool branches are §20 drops (SE-V1).
//!
//! The arity-overloaded lookup getters (`SE_GetString`/`SE_GetFlags` pairs,
//! `SE_GetNumFlags`/`GetFlagName`/`GetFlagMask`) are `StringEdPackage` seam
//! methods on [`package`] (SE-D7/RULING 57), not free functions here.
//!
//! §20 dead surface (module-doc notes only, not ported):
//! - SE-V2: `GetNumStrings`/`SetReference`/`GetCurrentFileName` are declared but
//!   never defined or called (`stringed_ingame.cpp:109,111,113`).
//! - SE-V6: the `Language_Is{Russian,Polish,…}` inline header helpers
//!   (`stringed_ingame.h:71-104`) have zero callers in the DEDICATED/WinDed link
//!   set (only the renderer/font `Language_IsAsian`, a separate trap, exists);
//!   retained for a future client/renderer wave.
//!
//! Source: `oracle/codemp/qcommon/stringed_ingame.h:1-120`,
//! `oracle/codemp/qcommon/stringed_ingame.cpp`,
//! `oracle/codemp/qcommon/stringed_interface.cpp`

pub mod api;
pub mod entry;
pub mod interface;
pub mod package;

pub use api::{SE_GetString, SE_GetString2};
pub use entry::SeEntry;
pub use package::StringEdPackage;

// --- SE_* text-equates (`stringed_ingame.h:10-34`) ---
// Hungarian value prefixes (`i`/`s`) drop per RULING 40; the meaningful `SE_`
// namespace prefix is kept.

/// Raven `iSE_VERSION` — the `.str`/`.ste` file-format version.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:22`
pub const SE_VERSION: i32 = 1;

/// Raven `iSE_MAX_FILENAME_LENGTH` (= `MAX_QPATH`) — the `Filename_*` scratch width.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:13`
pub const SE_MAX_FILENAME_LENGTH: usize = mp_qshared::shared::MAX_QPATH;

/// Raven `sSE_STRINGS_DIR` — the localized-strings root directory.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:14`
pub const SE_STRINGS_DIR: &str = "strings";

/// Raven `sSE_DEBUGSTR_PREFIX` — prefix onto debug versions of strings.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:15`
pub const SE_DEBUGSTR_PREFIX: &str = "[";

/// Raven `sSE_DEBUGSTR_SUFFIX`.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:16`
pub const SE_DEBUGSTR_SUFFIX: &str = "]";

/// Raven `sSE_KEYWORD_VERSION`.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:23`
pub const SE_KEYWORD_VERSION: &str = "VERSION";

/// Raven `sSE_KEYWORD_CONFIG`.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:24`
pub const SE_KEYWORD_CONFIG: &str = "CONFIG";

/// Raven `sSE_KEYWORD_FILENOTES`.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:25`
pub const SE_KEYWORD_FILENOTES: &str = "FILENOTES";

/// Raven `sSE_KEYWORD_REFERENCE`.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:26`
pub const SE_KEYWORD_REFERENCE: &str = "REFERENCE";

/// Raven `sSE_KEYWORD_FLAGS`.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:27`
pub const SE_KEYWORD_FLAGS: &str = "FLAGS";

/// Raven `sSE_KEYWORD_NOTES`.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:28`
pub const SE_KEYWORD_NOTES: &str = "NOTES";

/// Raven `sSE_KEYWORD_LANG` — the `LANG_<language>` line prefix.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:29`
pub const SE_KEYWORD_LANG: &str = "LANG_";

/// Raven `sSE_KEYWORD_ENDMARKER`.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:30`
pub const SE_KEYWORD_ENDMARKER: &str = "ENDMARKER";

/// Raven `sSE_FILE_EXTENSION` — editor-only, never used in-game.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:31`
pub const SE_FILE_EXTENSION: &str = ".st";

/// Raven `sSE_EXPORT_FILE_EXTENSION` — the `.ste` override file extension.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:32`
pub const SE_EXPORT_FILE_EXTENSION: &str = ".ste";

/// Raven `sSE_INGAME_FILE_EXTENSION` — the `.str` in-game file extension.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:33`
pub const SE_INGAME_FILE_EXTENSION: &str = ".str";

/// Raven `sSE_EXPORT_SAME` — the "reuse the english text" sentinel.
/// Source: `oracle/codemp/qcommon/stringed_ingame.h:34`
pub const SE_EXPORT_SAME: &str = "#same";
