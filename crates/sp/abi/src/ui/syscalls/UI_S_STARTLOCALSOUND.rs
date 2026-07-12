use core::ffi::c_int;

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_S_STARTLOCALSOUND` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:184`
pub struct UiSStartlocalsound;

impl OutboundSysCall for UiSStartlocalsound {
    type Import = SpUiImport;
    /// Args source: `oracle/code/ui/ui_local.h:2225` (C ABI for `trap_S_StartLocalSound`)
    /// Output source: `oracle/codemp/client/cl_ui.cpp:1003-1004` and `oracle/codemp/ui/ui_syscalls.c:210-211`
    /// TODO: SP transport path does not provide a direct `UI_S_STARTLOCALSOUND` case in `oracle/code/client/cl_ui.cpp`.
    type Args = (c_int, c_int);
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_S_STARTLOCALSOUND;
}
