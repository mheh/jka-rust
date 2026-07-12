//! Raven's `null` GL-implementation stubs — the DEDICATED/no-renderer build's
//! `GLimp_*`/`QGL_*` entry points, every body an intentional no-op.
//!
//! Source: `oracle/codemp/null/null_glimp.cpp`

use std::os::raw::c_char;

use mp_qshared::shared::{qboolean, qtrue};

/// Raven `GLimp_EndFrame`.
///
/// Source: `oracle/codemp/null/null_glimp.cpp:52-53`
pub fn GLimp_EndFrame() {}

/// Raven `GLimp_Init`.
///
/// Source: `oracle/codemp/null/null_glimp.cpp:55-57`
pub fn GLimp_Init() {}

/// Raven `GLimp_Shutdown`.
///
/// Source: `oracle/codemp/null/null_glimp.cpp:59-60`
pub fn GLimp_Shutdown() {}

/// Raven `GLimp_EnableLogging`.
///
/// Source: `oracle/codemp/null/null_glimp.cpp:62-63`
pub fn GLimp_EnableLogging(enable: qboolean) {
    let _ = enable;
}

/// Raven `GLimp_LogComment`.
///
/// Source: `oracle/codemp/null/null_glimp.cpp:65-66`
pub fn GLimp_LogComment(comment: *mut c_char) {
    let _ = comment;
}

/// Raven `QGL_Init`.
///
/// Raven: `return qtrue;`
/// Source: `oracle/codemp/null/null_glimp.cpp:68-70`
pub fn QGL_Init(dllname: *const c_char) -> qboolean {
    let _ = dllname;
    qtrue
}

/// Raven `QGL_Shutdown`.
///
/// Source: `oracle/codemp/null/null_glimp.cpp:72-73`
pub fn QGL_Shutdown() {}
