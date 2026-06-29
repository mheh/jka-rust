use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use crate::abi::generic::OutboundSysCall;

/// `UI_S_REGISTERSOUND` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:183`
pub struct UiSRegistersound;

impl OutboundSysCall for UiSRegistersound {
    type Import = SpUiImport;
    /// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:214-215`
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1000-1001`
    /// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1000-1001`
    /// SP `cl_ui.cpp` does not currently expose a dedicated `UI_S_REGISTERSOUND` case.
    type Args = *const c_char;
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_S_REGISTERSOUND;
}
