//! `Journal` — MP-only event journaling state (LIFE § State ownership).

/// The `Common.journal` field group (MP only). `file`/`data_file` are Raven
/// `fileHandle_t` (= `int` → `i32`, `q_shared.h:362`); `mode` is the `journal`
/// `CVAR_INIT` cvar (`common.cpp:761`).
///
/// Source: `oracle/codemp/qcommon/common.cpp:34-35`
pub struct Journal {
    pub file: i32,
    pub data_file: i32,
    pub mode: i32,
}
