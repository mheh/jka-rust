use core::ffi::c_int;

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_R_DRAWSTRETCHPIC` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:179`
pub struct UiRDrawstretchpic;

impl OutboundSysCall for UiRDrawstretchpic {
    type Import = SpUiImport;
    /// Args source: `oracle/code/ui/ui_public.h:179`,
    /// `oracle/codemp/ui/ui_syscalls.c:194-195`
    /// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:984-986`
    type Args = (f32, f32, f32, f32, f32, f32, f32, f32, c_int);
    /// Output source: `oracle/code/client/cl_ui.cpp` has no SP case for
    /// `UI_R_DRAWSTRETCHPIC`; fallback transport evidence: `oracle/codemp/client/cl_ui.cpp:984-986`
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_R_DRAWSTRETCHPIC;
}
