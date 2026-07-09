use core::ffi::c_void;

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_R_ADDREFENTITYTOSCENE` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:174`
pub struct UiRAddrefentitytoscene;

impl OutboundSysCall for UiRAddrefentitytoscene {
    type Import = SpUiImport;
    /// Args source: `oracle/code/ui/ui_public.h:174`,
    /// `oracle/codemp/ui/ui_syscalls.c:174-175`
    /// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:960-961`
    type Args = *const c_void;
    /// Output source: `oracle/code/client/cl_ui.cpp` has no SP case for
    /// `UI_R_ADDREFENTITYTOSCENE`; fallback transport evidence: `oracle/codemp/client/cl_ui.cpp:960-961`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_R_ADDREFENTITYTOSCENE;
}
