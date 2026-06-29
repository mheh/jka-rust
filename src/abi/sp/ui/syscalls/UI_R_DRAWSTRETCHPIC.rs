use core::ffi::c_int;

use super::super::SpUiImport;
use crate::abi::generic::OutboundSysCall;

/// `UI_R_DRAWSTRETCHPIC` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:179`
pub struct UiRDrawstretchpic;

impl OutboundSysCall for UiRDrawstretchpic {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/code/ui/ui_public.h:179`,
    /// `oracle/oracle/codemp/ui/ui_syscalls.c:194-195`
    /// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:984-986`
    type Args = (f32, f32, f32, f32, f32, f32, f32, f32, c_int);
    /// Output source: `oracle/oracle/code/client/cl_ui.cpp` has no SP case for
    /// `UI_R_DRAWSTRETCHPIC`; fallback transport evidence: `oracle/oracle/codemp/client/cl_ui.cpp:984-986`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_R_DRAWSTRETCHPIC;
}
