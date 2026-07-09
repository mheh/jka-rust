//! `ModuleSearchPolicy` — a mode's per-load module search policy (LOAD-D1/D9).

use super::naming::ModuleNaming;
use super::search_step::SearchStep;

/// A mode's search policy — a value, built **per load by the caller in
/// `mp_engine_qcommon`** (LOAD-D9), never by `native/platform` (which stays
/// cvar-free, porting-rules §B3). All paths are already resolved when built.
///
/// Source: `oracle/codemp/win32/win_main.cpp:811-887` (`Sys_LoadDll`).
pub struct ModuleSearchPolicy {
    pub naming: ModuleNaming,
    /// Bare `LoadLibrary(filename)` / CWD-default probe tried first (MP Win32
    /// only, `win_main.cpp:855`; Unix MP `#if 0`s its cwd dlopen,
    /// `unix_main.c:361-373`, so its policy sets this `false`).
    pub direct_first: bool,
    /// Ordered probes after the direct one; first hit wins.
    pub steps: Vec<SearchStep>,
}
