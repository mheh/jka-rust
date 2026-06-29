use core::ffi::c_int;

use super::super::SpUiImport;
use crate::abi::generic::OutboundSysCall;

/// `UI_R_MODELBOUNDS` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:208`
pub struct UiRModelbounds;

impl OutboundSysCall for UiRModelbounds {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/code/ui/ui_public.h:208`,
    /// `oracle/oracle/codemp/ui/ui_syscalls.c:198-199`
    /// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:988-990`
    type Args = (c_int, *mut f32, *mut f32);
    /// Output source: `oracle/oracle/code/client/cl_ui.cpp` does not include a SP case for
    /// `UI_R_MODELBOUNDS`; fallback transport evidence: `oracle/oracle/codemp/client/cl_ui.cpp:988-990`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_R_MODELBOUNDS;
}
