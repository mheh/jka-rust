use core::ffi::c_char;

use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_R_REMAP_SHADER` SP UI imports syscall boundary token.
///
/// Raven: 80
/// Source: `oracle/oracle/code/ui/ui_public.h:232`
pub struct UiRRemapShader;

impl OutboundSysCall for UiRRemapShader {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:1201-1203`.
    type Args = (*const c_char, *const c_char, *const c_char);
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1201-1203`.
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_R_REMAP_SHADER;
}
