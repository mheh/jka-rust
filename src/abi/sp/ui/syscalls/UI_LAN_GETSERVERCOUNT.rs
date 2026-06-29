use core::ffi::c_int;

use super::super::SpUiImport;
use crate::abi::generic::OutboundSysCall;

/// `UI_LAN_GETSERVERCOUNT` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:217`
pub struct UiLanGetservercount;

impl OutboundSysCall for UiLanGetservercount {
    type Import = SpUiImport;
    /// Raven wrapper: `return syscall( UI_LAN_GETSERVERCOUNT, source );`
    /// Transport source: `oracle/oracle/codemp/client/cl_ui.cpp:1085-1087`
    /// SP transport path in `oracle/oracle/code/client/cl_ui.cpp` has no `UI_LAN_GETSERVERCOUNT` case.
    ///
    /// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:1085-1087`
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1085-1087`
    type Args = c_int;
    /// Returns server count as integer.
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1085-1087`
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_GETSERVERCOUNT;
}
