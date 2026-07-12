use core::ffi::{c_int, c_void};

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_R_ADDPOLYTOSCENE` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:175`
pub struct UiRAddpolytoscene;

impl OutboundSysCall for UiRAddpolytoscene {
    type Import = SpUiImport;
    /// Args source: `oracle/code/ui/ui_public.h:175`,
    /// `oracle/codemp/ui/ui_syscalls.c:178-179`
    /// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:964-966`
    type Args = (c_int, c_int, *const c_void);
    /// Output source: `oracle/code/client/cl_ui.cpp` has no SP `UI_R_ADDPOLYTOSCENE` case;
    /// fallback transport evidence: `oracle/codemp/client/cl_ui.cpp:964-966`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_R_ADDPOLYTOSCENE;
}
