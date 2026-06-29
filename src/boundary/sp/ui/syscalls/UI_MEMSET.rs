use core::ffi::{c_int, c_void};

use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_MEMSET` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:239`
///
/// Args source: no SP `UI_MEMSET` case appears in
/// `oracle/oracle/code/client/cl_ui.cpp`; fallback transport shape from shared MP-side memory-call
/// handling: `oracle/oracle/codemp/client/cl_ui.cpp:818-819` (`TRAP_MEMSET`
/// receives `(char *)VMA(1), args[2], args[3]`).
///
/// Output source: shared MP-side `TRAP_MEMSET` handler also returns `0`
/// (`oracle/oracle/codemp/client/cl_ui.cpp:820`).
pub struct UiMemset;

impl OutboundSysCall for UiMemset {
    type Import = SpUiImport;
    /// `Com_Memset( VMA(1), args[2], args[3] )` equivalent payload.
    type Args = (*mut c_void, c_int, c_int);
    /// `TRAP_MEMSET` returns no payload (`void`).
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_MEMSET;
}
