use core::ffi::c_int;

use super::super::SpUiImport;
use crate::abi::generic::OutboundSysCall;

/// `UI_MILLISECONDS` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:154`
pub struct UiMilliseconds;

impl OutboundSysCall for UiMilliseconds {
    type Import = SpUiImport;
    /// `case UI_MILLISECONDS: return Sys_Milliseconds();`
    /// (`oracle/oracle/code/client/cl_ui.cpp:452`).
    ///
    /// Args source: none (token-only call).
    type Args = ();
    /// Returns the same `int`/`Sys_Milliseconds` payload as in SP UI switch.
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_MILLISECONDS;
}
