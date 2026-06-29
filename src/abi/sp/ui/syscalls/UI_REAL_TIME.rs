use core::ffi::c_int;

use super::super::SpUiImport;
use crate::codemp::game::q_shared_h::qtime_t;
use crate::abi::generic::OutboundSysCall;

/// `UI_REAL_TIME` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:216`
pub struct UiRealTime;

impl OutboundSysCall for UiRealTime {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/code/ui/ui_public.h:216`,
    /// `oracle/oracle/codemp/ui/ui_syscalls.c:400`
    /// Output/source switch evidence: `oracle/oracle/codemp/client/cl_ui.cpp:1180-1181`
    type Args = *mut qtime_t;
    /// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:400-401`;
    /// SP-side fallback evidence: `oracle/oracle/codemp/client/cl_ui.cpp:1180-1181`
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_REAL_TIME;
}
