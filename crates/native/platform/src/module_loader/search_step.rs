//! `SearchStep` — one ordered probe in a module search policy (LOAD-D1).

use std::path::PathBuf;

/// One ordered search probe; first hit wins. Carries **resolved** values
/// (LOAD-D9), never cvar names — the caller in `mp_engine_qcommon` reads the
/// `fs_*` cvars and plants the results, so `native/platform` never touches a
/// cvar table.
///
/// Source: `oracle/codemp/qcommon/files.cpp:479-498` (`FS_BuildOSPath`).
pub enum SearchStep {
    /// `FS_BuildOSPath(base, gamedir, filename)` → `"<base>/<gamedir>/<file>"`.
    /// An empty-`fs_cdpath` step is omitted by the caller (LOAD-D9 round-3), so
    /// every step handed to `sys_load_dll` is real and it walks them blindly.
    ///
    /// Source: `oracle/codemp/win32/win_main.cpp:858-869`
    FsPath { base: PathBuf, gamedir: String },
    // NOTE: the SP `CwdRelative { subdir }` variant (win_main.cpp:515,524) is
    // dropped — our SP constructs no policy (LOAD-D1 / LOAD-D5 / DEC-07), so it
    // was zero-constructor surface (porting-rules §20).
}
