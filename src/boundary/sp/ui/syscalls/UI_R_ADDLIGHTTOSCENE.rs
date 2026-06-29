use core::ffi::c_int;

use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_ADDLIGHTTOSCENE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:176`
pub struct UiRAddlighttoscene;

impl OutboundSysCall for UiRAddlighttoscene {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/code/ui/ui_public.h:176`,
    /// `oracle/oracle/codemp/ui/ui_syscalls.c:182-183`
    /// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:968-973`
    type Args = (*const f32, f32, f32, f32, f32);
    /// Output source: `oracle/oracle/code/client/cl_ui.cpp` does not include a SP UI case for
    /// `UI_R_ADDLIGHTTOSCENE`; fallback transport evidence: `oracle/oracle/codemp/client/cl_ui.cpp:968-973`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_R_ADDLIGHTTOSCENE;
}
